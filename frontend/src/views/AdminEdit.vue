<template>
    <h1>编辑文章</h1>
    <div v-if="loading">加载中</div>
    <ArticleForm v-else :isEdit="true" :articleId="id" :article="article" />
</template>

<script setup lang="ts">
import { getArticle } from "@/api/articles";
import ArticleForm from '@/components/article/ArticleForm.vue';
import type { Article } from '@/types/article';
import { ref } from 'vue';
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute()
const article = ref()
const loading = ref(true)
const id = ref(route.params.id as Article["id"])

const init = async () => {
    try {
        const res = await getArticle(route.params.id as Article["id"])
        // console.log(res.data.id)
        article.value = res.data
    } catch (e) {
        console.error('加载失败', e)
    } finally {
        loading.value = false
    }

}

onMounted(init)
</script>
