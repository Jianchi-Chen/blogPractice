<template>
    <n-card class="fav-card" size="large" title="收藏的文章" bordered>
        <div class="list-controls">
            <n-input
                size="small"
                clearable
                v-model:value="filter"
                placeholder="搜索文章标题或标签"
                style="max-width: 320px"
            />
            <n-button size="small" @click="refreshList">刷新</n-button>
        </div>

        <n-divider />

        <div v-if="loading" class="loading-wrap">
            <n-spin size="large" />
        </div>

        <div v-else>
            <n-list bordered>
                <n-list-item
                    v-for="item in filteredFavorites"
                    :key="item.id"
                    class="fav-item"
                >
                    <div class="item-left">
                        <div class="item-title">
                            {{ item.title }}
                        </div>
                        <div class="item-desc">
                            {{ item.excerpt }}
                        </div>
                        <div class="item-meta">
                            <n-tag v-for="tag in item.tags" :key="tag" small>{{
                                tag
                            }}</n-tag>
                            <span class="spacer"></span>
                            <span class="date">{{
                                formatDate(item.updatedAt)
                            }}</span>
                        </div>
                    </div>

                    <div class="item-actions">
                        <n-button size="tiny" @click="openArticle(item)"
                            >查看</n-button
                        >
                        <n-button
                            size="tiny"
                            secondary
                            @click="unfavorite(item.id)"
                            >取消收藏</n-button
                        >
                    </div>
                </n-list-item>
            </n-list>

            <div v-if="filteredFavorites.length === 0" class="empty-wrap">
                <n-empty description="没有找到符合条件的文章" />
            </div>
        </div>
    </n-card>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useUserStore } from "@/stores/user";
import { userStorageKey } from "@/utils/userStorage";

export interface FavoriteArticle {
    id: string;
    title: string;
    excerpt: string;
    tags: string[];
    updatedAt: number;
    link?: string;
}

const loading = ref(true);
const favorites = ref<FavoriteArticle[]>([]);
const filter = ref("");
const userStore = useUserStore();
const favoritesKey = computed(() =>
    userStorageKey(userStore.id || "", "favorites")
);

const sampleData: FavoriteArticle[] = [
    {
        id: "a1",
        title: "用 Vue 构建现代博客的最佳实践",
        excerpt:
            "分享如何使用 Vue 3 + Naive UI 快速构建一个简洁且可维护的博客前端。",
        tags: ["Vue", "Naive UI", "前端"],
        updatedAt: Date.now() - 1000 * 60 * 60 * 24 * 2,
        link: "https://example.com/article/a1",
    },
    {
        id: "a2",
        title: "前端性能优化：实战指南",
        excerpt: "从资源加载到渲染优化，带你一步步提升页面表现。",
        tags: ["性能", "优化"],
        updatedAt: Date.now() - 1000 * 60 * 60 * 24 * 10,
        link: "https://example.com/article/a2",
    },
];

const loadFavorites = () => {
    loading.value = true;
    try {
        const cached = localStorage.getItem(favoritesKey.value);
        if (cached) {
            favorites.value = JSON.parse(cached);
        } else {
            favorites.value = sampleData;
            localStorage.setItem(
                favoritesKey.value,
                JSON.stringify(sampleData)
            );
        }
    } catch {
        favorites.value = [];
        localStorage.removeItem(favoritesKey.value);
    } finally {
        loading.value = false;
    }
};

const filteredFavorites = computed(() => {
    if (!filter.value) return favorites.value;
    const q = filter.value.toLowerCase();
    return favorites.value.filter(
        (i) =>
            i.title.toLowerCase().includes(q) ||
            i.tags.some((t) => t.toLowerCase().includes(q)) ||
            (i.excerpt && i.excerpt.toLowerCase().includes(q))
    );
});

const refreshList = () => {
    loadFavorites();
};

const openArticle = (item: FavoriteArticle) => {
    window.open(item.link || "#", "_blank");
};

const unfavorite = (id: string) => {
    favorites.value = favorites.value.filter((i) => i.id !== id);
    localStorage.setItem(favoritesKey.value, JSON.stringify(favorites.value));
};

const formatDate = (ts: number) => {
    const d = new Date(ts);
    return (
        d.toLocaleDateString() +
        " " +
        d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    );
};

watch(() => userStore.id, loadFavorites, { immediate: true });
</script>

<style scoped>
.fav-card .list-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
}

.loading-wrap {
    display: flex;
    justify-content: center;
    padding: 40px 0;
}

.fav-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 12px 8px;
}

.item-left {
    max-width: 72%;
}

.item-title {
    font-weight: 600;
    margin-bottom: 6px;
}

.item-desc {
    color: var(--n-typography-3);
    font-size: 13px;
    margin-bottom: 8px;
}

.item-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
}

.item-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-end;
}

.spacer {
    width: 10px;
}

.empty-wrap {
    padding: 24px 0;
    display: flex;
    justify-content: center;
}
</style>
