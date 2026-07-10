<template>
    <n-flex align="center" :size="12">
        <!-- 🔍 图标按钮 -->
        <n-button
            text
            :focusable="false"
            @click="toggleSearch"
            style="font-size: 20px"
        >
            <n-icon size="33">
                <SearchCircleOutline />
            </n-icon>
        </n-button>

        <n-flex>
            <n-popover
                trigger="manual"
                :show="showPopover"
                placement="bottom-start"
            >
                <template #trigger>
                    <!-- input触发 -->
                    <n-input
                        v-show="isExpanded"
                        v-model:value="keyword"
                        placeholder="搜索文章"
                        round
                        clearable
                        @keyup.enter="handleSearch"
                        @input="handleInput"
                    >
                        <template #suffix>
                            <n-spin v-if="loading" size="small" />
                        </template>
                    </n-input>
                </template>
                <!-- 列表 -->
                <n-list hoverable clickable>
                    <n-list-item v-for="sug in suggestions" :key="sug.id">
                        <n-thing @click="() => handleSelect(sug.id)">{{
                            sug.title
                        }}</n-thing>
                    </n-list-item>
                </n-list>
            </n-popover>
        </n-flex>
    </n-flex>
</template>

<script setup lang="ts">
import { h, onMounted, ref, render, type Ref } from "vue";
import { NButton, NInput, NIcon } from "naive-ui";
import { SearchCircleOutline } from "@vicons/ionicons5";
import { debounce } from "lodash-es";
import { useRouter } from "vue-router";
import { fetchSuggestions } from "@/api/article";
import type { ArticleSuggestion } from "@/types/article";
import { useSearchStore } from "@/stores/search";

/**
 * 1. isExpanded 控制搜索框展开/收起
 * 2. keyword 实时收集用户输入
 * 3. 搜索后通过 emit 把结果抛给父组件
 */

const showPopover = ref(false);
const isExpanded = ref(false);
const keyword = ref("");
const loading = ref(false);
const suggestions: Ref<ArticleSuggestion[]> = ref([]);
const router = useRouter();
const search = useSearchStore();
let abortController: AbortController | null = null; // 防抖；取消旧请求

// 生成建议项
const generateSuggestions = debounce(async () => {
    const q = keyword.value.trim();
    if (!q) {
        suggestions.value = [];
        return;
    }
    /* 取消上一次请求 */
    abortController?.abort();
    abortController = new AbortController();
    loading.value = true;

    try {
        const res = await fetchSuggestions(keyword.value);
        const data = res.data.item;
        console.log(res.data);

        suggestions.value = data.map((item: any) => ({
            title: item.title, // ⬅ 使用 render 函数
            id: item.id,
        }));
    } catch (err) {
        console.error("Failed to fetch suggestions:", err);
    } finally {
        loading.value = false;
    }

    loading.value = false;
}, 300);

/** 切换展开状态 */
const toggleSearch = () => {
    isExpanded.value = !isExpanded.value;
    // 如果收起，自动清空
    if (!isExpanded.value) {
        keyword.value = "";
        search.setCondition(keyword.value);
        suggestions.value = [];
        showPopover.value = false;
    }
};

// 输入时触发, 生成建议项
const handleInput = () => {
    if (showPopover.value === false) showPopover.value = true;
    if (keyword.value.trim() == "") showPopover.value = false;
    generateSuggestions();
};

// 回车搜索
const handleSearch = () => {
    console.log(keyword.value);

    // TODO 展示具有相关内容的文章
    search.setCondition(keyword.value);
    router.push("/");
    showPopover.value = false;
};

// 跳转对应文章页
const handleSelect = (id: string | undefined) => {
    if (!id) {
        return;
    }
    router.push(`/article/${id}`); //！ 只能跳一次
    toggleSearch();
    isExpanded.value = !isExpanded.value;
};
</script>
