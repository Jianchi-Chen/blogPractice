import { z } from "zod";

// Zod 是一个 TypeScript 友好的 schema 校验库，可以帮你声明文章结构并自动验证。
export const ArticleSchema = z.object({
  id: z.string().optional(),
  title: z.string().optional(), // 字符串最小长度为1
  content: z.string().optional(),
  summary: z.string().optional(),
  created_at: z.iso.datetime().optional(), // 推荐用 ISO 时间字符串
  update_at: z.iso.datetime().optional(),
  update_count: z.number().optional(),
  author_name: z.string().optional(),
  status: z
    .enum(["published", "draft", "archived"])
    .optional()
    .default("draft"), // 枚举，默认draft
  views: z.number().min(0).optional(), // 非负数
  tags: z.string().optional().default("Universal"), // 标签
});

export type Article = z.infer<typeof ArticleSchema>; // 自动推导类型

export interface ArticleListResponse {
  articles: Article[];
}

export interface ArticleSuggestion {
  id?: string;
  title?: string;
}

export interface SuggestionResponse {
  item: ArticleSuggestion[];
}

// 初始化Article
export const createEmptyArticle = (): Article => {
  return ArticleSchema.parse({
    title: "",
    content: "",
  });
};
