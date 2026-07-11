<script setup lang="ts">
import { RouterView } from "vue-router";
import NavBar from "./components/layout/NavBar.vue";
import {
    darkTheme,
    NConfigProvider,
    NLayout,
    NLayoutHeader,
    NLayoutContent,
    NLayoutSider,
} from "naive-ui";
import type { GlobalTheme } from "naive-ui";
import { defineComponent, onMounted, onUnmounted, ref } from "vue";
import Sider from "@/components/layout/Sider.vue";
import { useArticleStore } from "./stores/article";
import { BackToTop } from "@vicons/carbon";
import { listen } from "@tauri-apps/api/event";
import { useMessage } from "naive-ui";

type UpdateStatusPayload = {
    status: "checking" | "found" | "downloading" | "none" | "error" | "done";
    progress?: number;
    message?: string;
};

const articleStore = useArticleStore();

// 主题色切换, null 等于 light
const theme = ref<GlobalTheme | null>(null);
const toggleTheme = () => {
    theme.value = articleStore.osTheme === false ? darkTheme : null;
    articleStore.osTheme = !articleStore.osTheme;
};

// 更新事件监听组件（必须位于 n-message-provider 之内）
const UpdateStatusListener = defineComponent({
    name: "UpdateStatusListener",
    setup() {
        const naiveMessage = useMessage();
        let unlisten: (() => void) | undefined;

        onMounted(async () => {
            try {
                unlisten = await listen<UpdateStatusPayload>(
                    "update_status",
                    (event) => {
                        const {
                            status,
                            progress,
                            message: msg,
                        } = event.payload;
                        switch (status) {
                            case "checking":
                                naiveMessage.loading("正在检查更新");
                                break;
                            case "found":
                                naiveMessage.info("检查到更新，正在下载...");
                                break;
                            case "downloading":
                                if (typeof progress === "number") {
                                    naiveMessage.info(`下载进度 ${progress}%`);
                                }
                                break;
                            case "none":
                                naiveMessage.success(msg || "未发现新版本");
                                break;
                            case "done":
                                naiveMessage.success(
                                    msg || "更新下载完成，正在安装"
                                );
                                break;
                            case "error":
                                naiveMessage.error(msg || "检查或下载更新失败");
                                break;
                        }
                        console.log("update_status:", event.payload.status);
                    }
                );
            } catch (e) {
                console.error("监听更新事件失败", e);
            }
        });

        onUnmounted(() => {
            if (unlisten) unlisten();
        });

        return () => null;
    },
});
</script>

<template>
    <!-- 整个 Naive UI 的全局配置上下文,例如主题、语言、图标等 -->
    <n-config-provider :theme="theme">
        <!-- 模态框 -->
        <n-modal-provider>
            <!-- 对话框 -->
            <n-dialog-provider>
                <!-- 所有页面组件都能访问 useMessage() 提供的 API -->
                <n-message-provider>
                    <UpdateStatusListener />
                    <n-layout position="absolute" class="h-screen h-full">
                        <n-layout-header bordered>
                            <NavBar @toggleTheme="toggleTheme" />
                        </n-layout-header>

                        <!-- 百分之94高, 是除开NavBar组件的高度 -->
                        <n-layout has-sider class="h-[94%]">
                            <!-- 天坑：不加collapse-mode="width"的话图标无法展示 -->
                            <n-layout-sider
                                :collapsed="articleStore.expandFolder"
                                :width="240"
                                :collapsed-width="64"
                                :size="100"
                                @collapse="articleStore.expandFolder = true"
                                @expand="articleStore.expandFolder = false"
                                collapse-mode="width"
                                class="!mr-2"
                                bordered
                            >
                                <Sider />
                            </n-layout-sider>

                            <n-layout-content>
                                <router-view />

                                <n-back-top
                                    :bottom="140"
                                    :visibility-height="300"
                                >
                                    <n-icon-wrapper
                                        :size="45"
                                        :border-radius="18"
                                    >
                                        <n-icon size="40">
                                            <BackToTop />
                                        </n-icon>
                                    </n-icon-wrapper>
                                </n-back-top>
                            </n-layout-content>
                        </n-layout>
                    </n-layout>
                </n-message-provider>
            </n-dialog-provider>
        </n-modal-provider>
    </n-config-provider>
</template>
