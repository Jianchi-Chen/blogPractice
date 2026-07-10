<template>
    <n-flex vertical :size="12">
        <n-flex>
            <n-button type="primary" @click="handleAdd">
                {{ newButton }}
            </n-button>
            <n-button @click="clearSorter"> Clear Sorter </n-button>
        </n-flex>

        <!-- 标签页 -->
        <n-card>
            <n-tabs v-model:value="activeTab" @update:value="changeNewButton">
                <n-tab-pane name="articleAdmin" tab="文章管理">
                    <ArticleManagement
                        ref="articleTableRef"
                        :articles="articles"
                        @refresh="loadArticlesAndUsers"
                    />
                </n-tab-pane>

                <n-tab-pane name="userAdmin" tab="用户管理">
                    <UserManagement
                        ref="userTableRef"
                        :users="users"
                        @refresh="loadArticlesAndUsers"
                        @edit="handleEditUser"
                    />
                </n-tab-pane>
            </n-tabs>
        </n-card>

        <NewUserDialog
            v-model:show="showNewUserDialog"
            @success="handlerUserSuccess"
        />

        <EditUserDialog
            v-model:show="showEditUserDialog"
            v-model:userdata="SelectedUserdata"
            @success="handlerUserSuccess"
        />
    </n-flex>
</template>

<script setup lang="ts">
import { NButton } from "naive-ui";
import { fetchArticles } from "@/api/article";
import { useUserStore } from "@/stores/user";
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { Article } from "@/types/article";
import type { User } from "@/types/user";
import { fetchUsers } from "@/api/account";
import NewUserDialog from "@/components/admin/NewUserDialog.vue";
import EditUserDialog from "@/components/admin/EditUserDialog.vue";
import ArticleManagement from "@/components/admin/ArticleManagement.vue";
import UserManagement from "@/components/admin/UserManagement.vue";

const router = useRouter();
const userStore = useUserStore();
const articles = ref<Article[]>([]);
const users = ref<User[]>([]);
const newButton = ref("New Article");
const activeTab = ref("articleAdmin");
const showNewUserDialog = ref(false);
const showEditUserDialog = ref(false);
const SelectedUserdata = ref<User | null>(null);
const articleTableRef = ref();
const userTableRef = ref();

// 页面加载时检查用户权限并加载数据
onMounted(() => {
    if (!userStore.isAdmin()) {
        router.push("/");
        return;
    }
    loadArticlesAndUsers();
});

// 加载数据
const loadArticlesAndUsers = async () => {
    const res = await fetchArticles();
    articles.value = res.data.articles;

    const resUsers = await fetchUsers(20);
    users.value = resUsers.data.users;
};

// 新建文章/用户
const handleAdd = () => {
    if (activeTab.value === "userAdmin") {
        showNewUserDialog.value = true;
        return;
    }
    router.push("/admin/createArticle");
};

// 编辑用户
const handleEditUser = (user: User) => {
    SelectedUserdata.value = user;
    showEditUserDialog.value = true;
};

// 清空排序
const clearSorter = () => {
    if (activeTab.value === "articleAdmin") {
        articleTableRef.value?.clearSorter();
    } else {
        userTableRef.value?.clearSorter();
    }
};

const changeNewButton = (value: string) => {
    newButton.value = value === "userAdmin" ? "New User" : "New Article";
};

// 新建或更新用户信息成功后的回调
const handlerUserSuccess = () => {
    showNewUserDialog.value = false;
    showEditUserDialog.value = false;
    loadArticlesAndUsers();
};
</script>
