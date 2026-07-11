const configuredApiUrl = import.meta.env.VITE_API_BASE_URL?.trim();

export const API_BASE_URL = (
    configuredApiUrl || (import.meta.env.DEV ? "http://127.0.0.1:3000" : "")
).replace(/\/+$/, "");

export const resolveApiUrl = (path?: string | null) => {
    if (!path || /^(?:https?:|data:|blob:)/i.test(path)) return path || "";
    if (!API_BASE_URL) return path.startsWith("/") ? path : `/${path}`;
    return `${API_BASE_URL}/${path.replace(/^\/+/, "")}`;
};
