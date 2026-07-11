<template>
    <n-card class="favorite-card" size="large" title="收藏的文章" bordered>
        <div class="list-controls">
            <n-input
                v-model:value="filter"
                size="small"
                clearable
                placeholder="搜索文章标题、摘要或标签"
            />
            <n-button size="small" :loading="loading" @click="loadFavorites">
                刷新
            </n-button>
        </div>

        <n-divider />

        <n-spin :show="loading">
            <n-list v-if="filteredFavorites.length" bordered>
                <n-list-item
                    v-for="article in filteredFavorites"
                    :key="article.id"
                    class="favorite-item"
                >
                    <div class="item-content">
                        <strong>{{ article.title }}</strong>
                        <n-text depth="3">{{ article.summary || "暂无摘要" }}</n-text>
                        <div class="item-meta">
                            <n-tag v-if="article.tags" size="small">
                                {{ article.tags }}
                            </n-tag>
                            <span>{{ formatDate(article.created_at) }}</span>
                        </div>
                    </div>

                    <template #suffix>
                        <n-space>
                            <n-button size="tiny" @click="openArticle(article.id)">
                                查看
                            </n-button>
                            <n-button
                                size="tiny"
                                secondary
                                type="error"
                                :loading="removingId === article.id"
                                :disabled="removingId !== null"
                                @click="unfavorite(article.id)"
                            >
                                取消收藏
                            </n-button>
                        </n-space>
                    </template>
                </n-list-item>
            </n-list>
            <n-empty v-else description="暂无收藏文章" />
        </n-spin>
    </n-card>
</template>

<script setup lang="ts">
import { getFavorites, removeFavorite } from "@/api/favorites";
import type { Article } from "@/types/article";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";

const router = useRouter();
const message = useMessage();
const loading = ref(false);
const removingId = ref<string | null>(null);
const favorites = ref<Article[]>([]);
const filter = ref("");
let latestLoadRequest = 0;

const filteredFavorites = computed(() => {
    const query = filter.value.trim().toLocaleLowerCase();
    if (!query) return favorites.value;
    return favorites.value.filter((article) =>
        [article.title, article.summary, article.tags].some((value) =>
            value?.toLocaleLowerCase().includes(query)
        )
    );
});

const loadFavorites = async () => {
    const requestId = ++latestLoadRequest;
    loading.value = true;
    try {
        const response = await getFavorites();
        if (requestId !== latestLoadRequest) return;
        favorites.value = response.data.articles;
    } catch (error) {
        if (requestId !== latestLoadRequest) return;
        favorites.value = [];
        console.error("Failed to load favorites", error);
        message.error("收藏列表加载失败");
    } finally {
        if (requestId === latestLoadRequest) loading.value = false;
    }
};

const openArticle = (id?: string) => {
    if (id) router.push({ name: "ArticleDetail", params: { id } });
};

const unfavorite = async (id?: string) => {
    if (!id || removingId.value) return;
    removingId.value = id;
    try {
        await removeFavorite(id);
        favorites.value = favorites.value.filter(
            (article) => article.id !== id
        );
        message.success("已取消收藏");
    } catch (error) {
        console.error("Failed to remove favorite", error);
        message.error("取消收藏失败");
    } finally {
        removingId.value = null;
    }
};

const formatDate = (value?: string) => {
    if (!value) return "";
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString("zh-CN");
};

onMounted(loadFavorites);
</script>

<style scoped>
.list-controls {
    display: grid;
    grid-template-columns: minmax(0, 320px) auto;
    justify-content: space-between;
    gap: 12px;
}

.favorite-item {
    padding: 12px 8px;
}

.item-content {
    display: grid;
    gap: 6px;
    min-width: 0;
}

.item-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--n-text-color-3);
    font-size: 12px;
}

@media (max-width: 640px) {
    .list-controls {
        grid-template-columns: 1fr;
    }
}
</style>
