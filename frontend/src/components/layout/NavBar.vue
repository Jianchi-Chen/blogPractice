<template>
    <n-flex justify="space-between" align="center" style="position: relative">
        <n-flex align="center">
            <n-button
                :focusable="false"
                style="font-size: 20px"
                :text="false"
                @click="handlerFolderExpand"
            >
                <n-icon size="27">
                    <FolderOpenOutline />
                </n-icon>
            </n-button>
            <n-switch @click="submitToggleTheme">
                <template #checked> Dark </template>
                <template #unchecked> Light </template>
            </n-switch>
            <!-- 搜索框 -->
            <NavSearch />
        </n-flex>

        <n-time
            time-zone="Asia/Shanghai"
            format="yyyy-MM-dd hh:mm"
            style="
                font-weight: bold;
                position: absolute;
                left: 50%;
                transform: translateX(-50%);
            "
        />

        <n-flex>
            <n-menu
                :options="menuOptions"
                v-model:value="activeKey"
                mode="horizontal"
                @update:value="handleSelect"
                responsive
            />
        </n-flex>
    </n-flex>
</template>

<script setup lang="ts">
import { NAvatar, NIcon, useMessage } from "naive-ui";
import { NMenu } from "naive-ui";
import { useUserStore } from "@/stores/user";
import { computed, h, ref } from "vue";
import { useRouter } from "vue-router";
import NavSearch from "@/components/layout/NavSearch.vue";
import { FolderOpenOutline } from "@vicons/ionicons5";
import { useArticleStore } from "@/stores/article";
import {
    HomeOutline,
    LogOutOutline,
    SettingsOutline,
    PersonOutline,
} from "@vicons/ionicons5";

const userstore = useUserStore();
const router = useRouter();
const message = useMessage();
const articleStore = useArticleStore();

// !!user.token 是布尔值，专门用来表示“是否登录”这个状态
const isLoggedin = computed(() => !!userstore.token);
const activeKey = ref();
const emit = defineEmits<{
    toggleTheme: [];
}>();

const submitToggleTheme = () => {
    // console.log("theme change");
    emit("toggleTheme");
};

// 利用computed(), 来创建基于其他响应式数据的派生值。当依赖变化时, 其会重新计算
const menuOptions = computed(() => {
    const items = [
        {
            label: "博客",
            key: "/",
            icon: render(HomeOutline),
        },
    ];

    if (isLoggedin.value) {
        if (!userstore.avatarUrl) {
            // 根据身份设定颜色
            const username = userstore.username || "G";
            const firstLetter = username.charAt(0).toUpperCase();

            let bg = "";
            let color = "white";

            if (userstore.isAdmin()) {
                bg = "red"; // 管理员：红底黄字
                color = "yellow";
            } else if (userstore.identity == "user") {
                bg = "green"; // 普通用户：绿底白字
                color = "white";
            } else {
                bg = "yellow"; // 游客：黄底黑字
                color = "black";
            }

            items.push({
                label: `当前用户: ${username}`,
                key: "/profile",
                icon: () =>
                    h(
                        NAvatar,
                        {
                            style: {
                                backgroundColor: bg,
                                color: color,
                                fontWeight: "bold",
                            },
                            size: 28,
                        },
                        { default: () => firstLetter }
                    ),
            });
        } else {
            items.push({
                label: `当前用户: ${userstore.username}`,
                key: "/profile",
                icon: () =>
                    h(NAvatar, {
                        size: 28,
                        src: userstore.avatarUrl,
                    }),
            });
        }

        if (userstore.isAdmin()) {
            items.push({
                label: "后台管理",
                key: "/admin",
                icon: render(SettingsOutline),
            });
        }

        items.push({
            label: "退出",
            key: "logout",
            icon: render(LogOutOutline),
        });
    } else {
        items.push({
            label: "登录",
            key: "/login",
            icon: render(PersonOutline),
        });
    }

    return items;
});

const handleSelect = (key: string) => {
    if (key == "logout") {
        userstore.logout();
        message.success("退出成功");
        router.push("/");
    } else {
        activeKey.value = key;
        router.push(`${key}`);
    }
};

// 添加图片函数
const render = (icon: any) => {
    return () => h(NIcon, null, { default: () => h(icon) });
};

// 控制侧边导航栏展开与否
const handlerFolderExpand = () => {
    articleStore.expandFolder = !articleStore.expandFolder;
};
</script>
