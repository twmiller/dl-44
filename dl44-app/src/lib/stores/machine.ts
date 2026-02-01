/**
 * Machine state stores for reactive UI updates.
 */
import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface Overrides {
  feed: number;
  rapid: number;
  spindle: number;
}

export interface Accessories {
  spindle_cw: boolean;
  spindle_ccw: boolean;
  flood_coolant: boolean;
  mist_coolant: boolean;
}

export type MachineState =
  | "idle"
  | "run"
  | "hold"
  | "jog"
  | "alarm"
  | "door"
  | "check"
  | "home"
  | "sleep"
  | "unknown";

export interface MachineStatus {
  state: MachineState;
  machine_pos: Position;
  work_pos: Position | null;
  work_offset: Position | null;
  feed_rate: number | null;
  spindle_speed: number | null;
  overrides: Overrides | null;
  input_pins: string | null;
  accessories: Accessories | null;
  buffer: [number, number] | null;
  line_number: number | null;
}

export type ConnectionState =
  | { Disconnected: null }
  | { Connecting: null }
  | { Connected: { port: string; baud: number } }
  | { Error: string };

export interface PortInfo {
  path: string;
  port_type: string;
  manufacturer: string | null;
  product: string | null;
  serial_number: string | null;
}

export interface ControllerSnapshot {
  connection: ConnectionState;
  status: MachineStatus;
  welcome_message: string | null;
  last_error: string | null;
}

/** Structured error from backend commands */
export interface CommandError {
  message: string;
  code: string;
  details: string | null;
}

/** UI-facing error with timestamp and dismissal */
export interface UIError {
  id: number;
  error: CommandError;
  timestamp: Date;
  dismissed: boolean;
}

// Helper to check connection state type
export function isConnected(state: ConnectionState): boolean {
  return "Connected" in state;
}

export function isDisconnected(state: ConnectionState): boolean {
  return "Disconnected" in state;
}

export function isConnecting(state: ConnectionState): boolean {
  return "Connecting" in state;
}

export function hasError(state: ConnectionState): boolean {
  return "Error" in state;
}

export function getConnectionInfo(
  state: ConnectionState
): { port: string; baud: number } | null {
  if ("Connected" in state) {
    return state.Connected;
  }
  return null;
}

/** Parse a backend error into a CommandError */
function parseError(e: unknown): CommandError {
  if (typeof e === "object" && e !== null) {
    const err = e as Record<string, unknown>;
    if (typeof err.message === "string") {
      return {
        message: err.message,
        code: typeof err.code === "string" ? err.code : "UNKNOWN",
        details: typeof err.details === "string" ? err.details : null,
      };
    }
  }
  return {
    message: String(e),
    code: "UNKNOWN",
    details: null,
  };
}

// Stores

/** Available serial ports */
export const ports = writable<PortInfo[]>([]);

/** Supported baud rates */
export const baudRates = writable<number[]>([115200]);

/** Selected port path */
export const selectedPort = writable<string>("");

/** Selected baud rate */
export const selectedBaud = writable<number>(115200);

/** Full controller snapshot */
export const controllerSnapshot = writable<ControllerSnapshot | null>(null);

/** Status polling interval ID */
let pollingInterval: ReturnType<typeof setInterval> | null = null;

/** Polling active flag */
export const isPolling = writable(false);

/** Error history (most recent first) */
let errorIdCounter = 0;
export const errors = writable<UIError[]>([]);

/** Most recent non-dismissed error */
export const lastError = derived(errors, ($errors) =>
  $errors.find((e) => !e.dismissed) ?? null
);

// Derived stores for convenience

export const connectionState = derived(
  controllerSnapshot,
  ($snapshot) => $snapshot?.connection ?? { Disconnected: null }
);

export const machineStatus = derived(
  controllerSnapshot,
  ($snapshot) => $snapshot?.status ?? null
);

export const machineState = derived(
  machineStatus,
  ($status) => $status?.state ?? "unknown"
);

export const machinePosition = derived(
  machineStatus,
  ($status) => $status?.machine_pos ?? { x: 0, y: 0, z: 0 }
);

export const workPosition = derived(
  machineStatus,
  ($status) => $status?.work_pos ?? $status?.machine_pos ?? { x: 0, y: 0, z: 0 }
);

export const connected = derived(connectionState, ($state) =>
  isConnected($state)
);

// Error management

/** Add an error to the error store */
export function addError(error: CommandError): UIError {
  const uiError: UIError = {
    id: ++errorIdCounter,
    error,
    timestamp: new Date(),
    dismissed: false,
  };

  errors.update((list) => {
    // Keep last 10 errors
    const updated = [uiError, ...list].slice(0, 10);
    return updated;
  });

  return uiError;
}

/** Dismiss an error by ID */
export function dismissError(id: number): void {
  errors.update((list) =>
    list.map((e) => (e.id === id ? { ...e, dismissed: true } : e))
  );
}

/** Clear all errors */
export function clearErrors(): void {
  errors.set([]);
}

/** Get user-friendly error message */
export function getErrorMessage(error: CommandError): string {
  switch (error.code) {
    case "TIMEOUT":
      return `Command timed out (${error.details ?? "no response from device"})`;
    case "GRBL_ERROR":
      return `GRBL error ${error.details ?? ""}: ${error.message}`;
    case "ALARM":
      return `Alarm ${error.details ?? ""}: Machine requires attention`;
    case "NOT_CONNECTED":
      return "Not connected to device";
    case "SERIAL_ERROR":
      return `Serial communication error: ${error.message}`;
    case "INVALID_STATE":
      return error.message;
    default:
      return error.message;
  }
}

// Actions

/** Refresh available serial ports */
export async function refreshPorts(): Promise<void> {
  try {
    const portList = await invoke<PortInfo[]>("list_serial_ports");
    ports.set(portList);

    // Auto-select first port if none selected
    const current = get(selectedPort);
    if (!current && portList.length > 0) {
      selectedPort.set(portList[0].path);
    }
  } catch (e) {
    console.error("Failed to list ports:", e);
    addError(parseError(e));
  }
}

/** Load supported baud rates */
export async function loadBaudRates(): Promise<void> {
  try {
    const rates = await invoke<number[]>("get_baud_rates");
    baudRates.set(rates);
  } catch (e) {
    console.error("Failed to get baud rates:", e);
  }
}

/** Connect to the selected port */
export async function connect(): Promise<void> {
  const port = get(selectedPort);
  const baud = get(selectedBaud);

  if (!port) {
    throw new Error("No port selected");
  }

  try {
    await invoke("connect", { port, baudRate: baud });
    await refreshSnapshot();
    startPolling();
  } catch (e) {
    const error = parseError(e);
    addError(error);
    await refreshSnapshot();
    throw error;
  }
}

/** Disconnect from the device */
export async function disconnect(): Promise<void> {
  stopPolling();
  try {
    await invoke("disconnect");
  } catch (e) {
    addError(parseError(e));
  }
  await refreshSnapshot();
}

/** Refresh controller snapshot */
export async function refreshSnapshot(): Promise<void> {
  try {
    const snapshot = await invoke<ControllerSnapshot>("get_controller_snapshot");
    controllerSnapshot.set(snapshot);
  } catch (e) {
    console.error("Failed to get snapshot:", e);
  }
}

/** Poll status from device */
export async function pollStatus(): Promise<void> {
  try {
    await invoke("poll_status");
    await refreshSnapshot();
  } catch (e) {
    // Don't spam errors for polling failures - they're expected during disconnects
    console.warn("Poll status failed:", e);
  }
}

/** Start automatic status polling */
export function startPolling(intervalMs: number = 250): void {
  stopPolling();
  isPolling.set(true);
  pollingInterval = setInterval(pollStatus, intervalMs);
}

/** Stop automatic status polling */
export function stopPolling(): void {
  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }
  isPolling.set(false);
}

// Control actions with error handling

/** Send home command */
export async function home(): Promise<void> {
  try {
    await invoke("home");
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
}

/** Send unlock command */
export async function unlock(): Promise<void> {
  try {
    await invoke("unlock");
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
}

/** Send jog command */
export async function jog(
  x: number | null,
  y: number | null,
  z: number | null,
  feed: number,
  incremental: boolean = true
): Promise<void> {
  try {
    await invoke("jog", { x, y, z, feed, incremental });
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
}

/** Cancel active jog */
export async function jogCancel(): Promise<void> {
  try {
    await invoke("jog_cancel");
  } catch (e) {
    // Jog cancel failures are usually benign
    console.warn("Jog cancel failed:", e);
  }
}

/** Send feed hold */
export async function feedHold(): Promise<void> {
  try {
    await invoke("feed_hold");
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
}

/** Send cycle start */
export async function cycleStart(): Promise<void> {
  try {
    await invoke("cycle_start");
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
}

/** Send soft reset */
export async function softReset(): Promise<void> {
  try {
    await invoke("soft_reset");
  } catch (e) {
    const error = parseError(e);
    addError(error);
    throw error;
  }
  await refreshSnapshot();
}

/** Initialize stores on app start */
export async function initializeStores(): Promise<void> {
  await Promise.all([refreshPorts(), loadBaudRates(), refreshSnapshot()]);
}
