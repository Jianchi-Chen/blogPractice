// src/mocks/handlers.ts 定义 mock 接口
import { getArticles } from "@/api/articles";
import { ArticleSchema, type Article } from "@/types/article";
import { http, HttpResponse } from "msw";
import { error } from "naive-ui/es/_utils/naive/warn";
import type { StringMappingType } from "typescript";
import { ref } from "vue";
import { useRouter } from "vue-router";
import { uuidv7 } from "uuidv7";
import z, { number, record, uuid } from "zod";
import { format } from "date-fns"; // 格式化展示
import type { User } from "@/types/user";
import type { Comment } from "@/types/comment";
import { useUserStore } from "@/stores/user";
import markdownContent from "./cite.md?raw";

export const handlers = [
  // /login 登录
  http.post("/login", async ({ request }) => {
    // 解包请求内容, 动态响应传入的用户名字符串
    const res = (await request.json()) as User;
    const exsitingAccount = users.value.find((e) => e.username == res.username);
    if (!exsitingAccount) {
      console.log("账户不存在", exsitingAccount);
      return HttpResponse.json({
        message: "failed",
      });
    }

    return HttpResponse.json({
      token: "mock-jwt-token",
      username: res.username,
    });
  }),

  // 获取文章列表
  http.get("/articles", ({ request }) => {
    // GET请求的参数在url中，所以要从url中拿
    const url = new URL(request.url); // 把字符串转换成 URL 对象
    const identity = url.searchParams.get("identity"); // ✅ 可以正常访问查询参数
    const condition = url.searchParams.get("condition");
    const res_array = ref<Article[]>([]);

    // 识别身份，输出
    if (identity === "admin") {
      res_array.value = articles.value;
    } else if (identity === "visitor") {
      res_array.value = articles.value.filter(
        (art) => art.status === "published"
      );
    } else {
      res_array.value = articles.value.filter(
        (art) => art.status === "published"
      );
    }

    // 识别条件
    if (condition !== "") {
      const tmp = condition?.toLowerCase() || ""; // 大小写不敏感

      res_array.value = res_array.value.filter((article) =>
        article.title?.toLowerCase().includes(tmp)
      );
    }

    // console.log(res_array.value);

    return HttpResponse.json(res_array.value);
  }),

  // 获取文章详情, 使用动态路由
  http.get("/article/:id", ({ params }) => {
    const { id } = params; // 解构 id
    const find_article = articles.value.find((a) => a.id === id);
    if (!find_article) {
      return HttpResponse.json({
        error: "文章不存在",
      });
    }

    return HttpResponse.json(find_article);
  }),

  // (新建)发布文章
  http.post("/article", async ({ request }) => {
    const json = await request.json(); // 未经验证的生肉
    const result = ArticleSchema.safeParse(json); // 验证，需要数据结构的值而非类型
    if (!result.success) {
      console.log(result.error.issues); // 查看未通过的具体属性
      return HttpResponse.json(
        { error: "title , content and summary can't empty" },
        { status: 400 }
      );
    }
    // 格式化时间
    const newArticle = {
      id: uuidv7(), // uuid生成唯一且有序的id
      ...result.data,
      created_at:
        result.data.created_at ??
        format(new Date().toISOString(), "yyyy-MM-dd HH:mm"), // nullish表达式，如果是空值就给一个时间
    };
    const formattedTime = format(new Date().toISOString(), "yyyy-MM-dd HH:mm");
    newArticle.created_at = formattedTime;

    // 添加默认简介
    if (!newArticle.summary) {
      newArticle.summary = result.data.title;
    }

    articles.value.push(newArticle);
    return HttpResponse.json(newArticle);
  }),

  // 删除文章
  http.delete("/article/:id", async ({ params }) => {
    const id = params.id as Article["id"];
    const index = articles.value.findIndex((a) => a.id === id);
    // console.log(index);
    if (index !== -1) {
      articles.value = articles.value.filter((article) => article.id !== id);
      return HttpResponse.json({ message: "删除成功" });
    }
    return HttpResponse.json({ message: "删除失败" });
  }),

  // 修改文章
  http.put("/article/:id", async ({ request, params }) => {
    const id = params.id as Article["id"];
    const res = (await request.json()) as Article;
    const index = articles.value.findIndex((a) => a.id === id);

    if (index !== -1 && typeof res == "object" && res !== null) {
      articles.value[index] = { ...articles.value[index], ...res };
    }

    return HttpResponse.json(articles.value[index]);
  }),

  // 更变文章状态
  http.patch("/article/:id", async ({ request, params }) => {
    const id = params.id as Article["id"];
    // 类型断言
    const data = (await request.json()) as { toggle?: string };
    const toggle = data.toggle as Article["status"];

    const index = articles.value.findIndex((a) => a.id == id);

    if (index !== -1 && articles.value[index].status !== toggle) {
      articles.value[index].status = toggle;
    }

    return HttpResponse.json(articles.value[index]);
  }),

  // 注册账户
  http.post("/registerAccount", async ({ request }) => {
    const res = (await request.json()) as User;
    const existingAccount = users.value.find((e) => e.username == res.username);
    if (existingAccount) {
      return HttpResponse.json({ message: "failed" });
    }
    res.id = uuidv7();
    users.value.push({ ...res });
    return HttpResponse.json({
      token: "mock-jwt-token",
      username: res.username,
    });
  }),

  // 获取评论
  http.get("/comments/:articleId", ({ params }) => {
    const id = params.articleId as string;
    return HttpResponse.json(comments[id] || []);
  }),

  // 发表评论
  http.post("/comment", async ({ request }) => {
    const res = (await request.json()) as {
      user: string;
      content: string;
      articleId: string;
    }; //断言成对象
    console.log(res.articleId);
    // const index = articles.value.findIndex((c) => c.id == res.articleId);
    // console.log(index);
    const newComment = {
      id: uuidv7(),
      user: res.user,
      content: res.content,
      created_at: format(new Date().toISOString(), "yyyy-MM-dd HH:mm"),
    };
    // 文章不存在就return,评论区存在就unshift,不存在就新建一个数组
    if (articles.value.findIndex((c) => c.id == res.articleId) == -1) {
      console.log("article isn't existed");
      return HttpResponse.json({ error: "文章不存在" }, { status: 404 });
    } else if (comments[res.articleId]) {
      comments[res.articleId].unshift(newComment);
    } else {
      comments[res.articleId] = [newComment];
    }
    return HttpResponse.json(newComment);
  }),

  // 删除评论
  http.delete("/comment", async ({ request }) => {
    const url = new URL(request.url);
    const CommentId = url.searchParams.get("CommentId");
    const ArticleId = url.searchParams.get("ArticleId") || "";
    console.log("h2", ArticleId);

    console.log("handler1: ", comments[ArticleId]);

    if (ArticleId) {
      comments[ArticleId] = comments[ArticleId].filter(
        (i) => i.id !== CommentId
      );
    }
    console.log("handler2: ", comments[ArticleId]);

    return HttpResponse.json(comments[ArticleId]);
  }),

  // 获取关键词搜索建议
  http.get("/suggestions/:keyword", ({ params }) => {
    const res = params.keyword as string;
    const sug = res.toLowerCase() || ""; // 大小写不敏感

    const suggestions = articles.value
      .filter((article) => article.title?.toLowerCase().includes(sug))
      .map((article) => ({ title: article.title, id: article.id }));

    return HttpResponse.json(suggestions);
  }),

  // 根据条件搜索
  http.get("/article/:condition", ({ params }) => {
    const res = params.condition as string;
    const lower_res = res.toLowerCase() || "";

    const afterFilter = articles.value.filter((art) =>
      art.title?.toLowerCase().includes(lower_res)
    );

    return HttpResponse.json(afterFilter);
  }),
];

// 模拟数据库，只存于内存中
let articles = ref<Article[]>([
  {
    id: "1",
    title: "Vue 3 从入门到实战",
    summary: "学习 Vue 3 的最佳实践",
    status: "published",
    created_at: "2024-06-01",
    content: markdownContent,
    tags: "Vue",
  },
  {
    id: "2",
    title: "深入理解 Pinia",
    summary: "下一代状态管理工具",
    status: "published",
    created_at: "2024-06-10",
    content: "最喜欢Pinia了",
    tags: "Pinia",
  },
  {
    id: "3",
    title: "Tailwind CSS 快速指南",
    summary: "从零上手到精通",
    status: "draft",
    created_at: "2024-06-15",
    content: "最喜欢Tailwind了",
    tags: "Tailwind",
  },
  // {
  //       id: "4",
  //   title: "Naive ui 布局四件套",
  //   summary: "从零上手到精通",
  //   status: "published",
  //   created_at: "2024-06-15",
  //   content: "",
  //   tags: ["Tailwind"],
  // },
]);

// 用户数据库
const users = ref<User[]>([
  {
    username: "123",
    identity: "admin",
    password: "123",
    id: "1",
    token: "",
  },
  {
    username: "u",
    identity: "user",
    password: "123",
    id: "2",
    token: "",
  },
  {
    username: "v",
    password: "123",
    identity: "vistor",
    id: "3",
    token: "",
  },
]);

// 评论数据库
const comments: Record<string, Comment[]> = {
  "1": [
    {
      id: "1",
      user: "123",
      content: "123 comment",
      created_at: "2024-06-10",
    },
  ],
};
