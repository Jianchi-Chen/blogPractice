import { fileURLToPath, URL } from "node:url";

import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import { loadEnv } from "vite";
import vueDevTools from "vite-plugin-vue-devtools";
import { defineConfig } from "vitest/config";

const normalizeApiBaseUrl = (value: string | undefined) => {
    const normalized = value?.trim().replace(/\/+$/, "") || "";
    if (!normalized) return "";

    const url = new URL(normalized);
    if (!["http:", "https:"].includes(url.protocol)) {
        throw new Error("VITE_API_BASE_URL must use HTTP or HTTPS");
    }
    if (url.pathname !== "/" || url.search || url.hash) {
        throw new Error("VITE_API_BASE_URL must be an origin without a path");
    }
    return url.origin;
};

export default defineConfig(({ command, mode }) => {
    const env = loadEnv(mode, process.cwd(), "");
    const apiBaseUrl = normalizeApiBaseUrl(env.VITE_API_BASE_URL);
    const isTauriBuild =
        command === "build" && Boolean(process.env.TAURI_ENV_PLATFORM);

    if (isTauriBuild && !apiBaseUrl) {
        throw new Error("VITE_API_BASE_URL is required for Tauri builds");
    }

    return {
        plugins: [
            vue(),
            vueDevTools(),
            tailwindcss(),
            Components({
                resolvers: [NaiveUiResolver()],
                dts: true,
            }),
        ],
        resolve: {
            alias: {
                "@": fileURLToPath(new URL("./src", import.meta.url)),
            },
        },
        server: {
            fs: {
                allow: [".."],
            },
            proxy: {
                "/api": {
                    target: apiBaseUrl || "http://127.0.0.1:3000",
                    changeOrigin: true,
                },
            },
        },
        optimizeDeps: {
            exclude: ["naive-ui"],
            include: ["@vicons/ionicons5", "@vicons/material", "@vicons/carbon"],
        },
        build: {
            commonjsOptions: {
                include: [/node_modules/],
            },
            rollupOptions: {
                maxParallelFileOps: 20,
            },
        },
        test: {
            environment: "jsdom",
            clearMocks: true,
            restoreMocks: true,
        },
        base: process.env.TAURI_ENV_PLATFORM ? "/" : "/myBlog/",
    };
});
