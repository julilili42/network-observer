export const DEFAULT_SERVER_PORT = 3000;
export const POLL_INTERVAL_MS = 3000;
export const MAX_EVENTS = 200;
export const MAX_FEED_VISIBLE = 60;
export const MAX_BUFFER_SIZE = 1000;

export function getApiBase(port: number): string {
  return `https://localhost:${port}`;
}

export function getWsUrl(port: number): string {
  return `wss://localhost:${port}/ws`;
}