import { createContext, useContext } from "react";
import type { ApiInstance } from "../utils/api";

export const ApiContext = createContext<ApiInstance | null>(null);

export function useApi(): ApiInstance {
  const ctx = useContext(ApiContext);
  if (!ctx) throw new Error("useApi must be used within ApiContext.Provider");
  return ctx;
}