import type { CurrentUser, User } from "@/types/user";
import { userStorageKey } from "@/utils/userStorage";
import { defineStore } from "pinia";

export const useUserStore = defineStore("user", {
    // state 本身是一个函数，调用useUserStore时会被初始化执行一次
    // 箭头函数的值返回给了pinia store的内部状态系统，成为这个store的默认数据源
    state: (): User => ({
        id: "",
        token: "", // 存储 JWT token
        username: "", // 当前登录用户
        password: "",
        identity: "", // 当前用户身份
        avatarUrl: "", // 用户头像 URL
    }),

    // 全局行为函数
    actions: {
        login(
            token: string,
            user: CurrentUser
        ) {
            this.token = token;
            localStorage.setItem("token", token);
            this.updateCurrentUser(user);
        },

        updateCurrentUser(user: CurrentUser) {
            this.username = user.username;
            this.identity = user.identity;
            this.id = user.id;
            this.avatarUrl =
                localStorage.getItem(userStorageKey(user.id, "avatar")) || "";
            localStorage.setItem("id", user.id);
            localStorage.setItem("username", user.username);
            localStorage.setItem("identity", user.identity);
        },

        logout() {
            this.token = "";
            this.username = "";
            this.identity = "";
            this.id = "";
            this.avatarUrl = "";
            localStorage.removeItem("token");
            localStorage.removeItem("username");
            localStorage.removeItem("identity");
            localStorage.removeItem("id");
        },

        initFromStorage() {
            this.token = localStorage.getItem("token") || "";
            this.username = localStorage.getItem("username") || "";
            this.identity = localStorage.getItem("identity") || "";
            this.id = localStorage.getItem("id") || "";
            this.avatarUrl = this.id
                ? localStorage.getItem(userStorageKey(this.id, "avatar")) || ""
                : "";
        },

        setAvatar(value: string) {
            this.avatarUrl = value;
            if (!this.id) return;

            const key = userStorageKey(this.id, "avatar");
            if (value) {
                localStorage.setItem(key, value);
            } else {
                localStorage.removeItem(key);
            }
        },

        // 判断当前用户是否是管理员
        isAdmin() {
            return this.identity === "admin";
        },
    },
});
