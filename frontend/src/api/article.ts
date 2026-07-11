import type {
    Article,
    ArticleListResponse,
    SuggestionResponse,
} from "@/types/article";
import client from "./client";
import { useAppStore } from "@/stores/app";
import { useUserStore } from "@/stores/user";
import { invoke } from "@tauri-apps/api/core";

// 封装api函数返回promis，故前端调用时使用async/await即可

// 获取文章列表
export const fetchArticles = async (condition?: string) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke<ArticleListResponse>("get_articles", {
            condition,
            token: user.token || null,
        });
        return { data };
    }
    
    // GET请求中，第二个参数需要写在params里，params 是专门用来指定 URL 查询参数的字段，它的值必须是一个对象
    return client.get<ArticleListResponse>("/articles", { params: { condition } });
};

// 获取文章详情
// 从表单、输入框获得的值通常为string, 所以id的类型设置为两种
// 模板字面量使用反斜杠``包裹，否则不识别！
export const fetchArticleById = async (id: Article["id"]) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke<Article>("get_article_by_id", {
            id,
            token: user.token || null,
        });
        return { data };
    }
    
    return client.get(`/article/${id}`);
};

// 新建文章
export const publishArticle = async (arg: Article) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke("create_article", { 
            token: user.token,
            articleData: arg 
        });
        return { data };
    }
    
    return client.post("/api/article", arg);
};

// 修改文章
export const updateArticle = async (id: Article["id"], data: Article) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const result = await invoke("update_article", { 
            token: user.token,
            id,
            articleData: data 
        });
        return { data: result };
    }
    
    return client.put(`/api/article/${id}`, data);
};

// 删除文章
export const deleteArticle = async (id: Article["id"]) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke("delete_article", { 
            token: user.token,
            id 
        });
        return { data };
    }
    
    return client.delete(`/api/article/${id}`);
};

// 转换文章状态
export const toggleStatus = async (id: Article["id"], toggle: string) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke("toggle_article_status", { 
            token: user.token,
            id,
            status: { toggle }  // 后端期望 NewStatus 结构体
        });
        return { data };
    }
    
    return client.patch(`/api/article/${id}`, { toggle });
};

// 获取建议
export const fetchSuggestions = async (
    keyword: string,
    signal?: AbortSignal
) => {
    const app = useAppStore();
    
    if (app.isTauri) {
        const data = await invoke<SuggestionResponse>("get_suggestions", { keyword });
        return { data };
    }
    
    return client.get<SuggestionResponse>(
        `/suggestions/${encodeURIComponent(keyword)}`,
        { signal }
    );
};

// 根据标签获取文章
export const fetchArticleByConditions = async (condition: string) => {
    return fetchArticles(condition);
};
