<template>
    <div class="profile-view">
        <n-grid :x-gap="24" :cols="3" :cols-sm="1">
            <n-grid-item :span="1">
                <UserProfile :user="user" />
            </n-grid-item>
            <n-grid-item :span="2">
                <FavoriteArticles />
            </n-grid-item>
        </n-grid>
    </div>
</template>

<script setup lang="ts">
import FavoriteArticles from "@/components/user/FavoriteArticles.vue";
import UserProfile from "@/components/user/UserProfile.vue";
import { useUserStore } from "@/stores/user";
import { computed } from "vue";

const userStore = useUserStore();
const user = computed(() => ({
    id: userStore.id || "-",
    name: userStore.username || "-",
}));
</script>

<style scoped>
.profile-view {
    padding: 20px;
}
</style>
