import { fileURLToPath, URL } from "node:url";

import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";
import vueDevTools from "vite-plugin-vue-devtools";
import tailwindcss from "@tailwindcss/vite";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd(), "");
    const apiTarget = (
        env.VITE_API_BASE_URL || "http://127.0.0.1:3000"
    ).replace(/\/+$/, "");

    return {
    plugins: [
        vue(),
        vueDevTools(),
        tailwindcss(),

        // 1. 自动引入组件
        Components({
            resolvers: [NaiveUiResolver()], // 关键
            dts: true, // 生成 .d.ts，TS 有提示
        }),
    ],
    resolve: {
        alias: {
            "@": fileURLToPath(new URL("./src", import.meta.url)),
        },
    },

    // 配置 Vite 代理
    server: {
        fs: {
            // 允许访问 node_modules 中的文件
            allow: [".."],
        },
        proxy: {
            "/api": {
                target: apiTarget,
                changeOrigin: true,
                rewrite: (path) => path,
            },
        },
    },

    // 避免 Vite 在构建时错误地处理 Naive UI 的某些模块。
    optimizeDeps: {
        exclude: ["naive-ui"],
        include: ["@vicons/ionicons5", "@vicons/material", "@vicons/carbon"],
    },
    build: {
        commonjsOptions: {
            include: [/node_modules/],
        },
        rollupOptions: {
            maxParallelFileOps: 20, // 限制并发文件操作数，解决 EMFILE 错误
        },
    },

    // Tauri 使用相对路径，Web 部署使用 /myBlog/
    base: process.env.TAURI_ENV_PLATFORM ? "/" : "/myBlog/",
    };
});
