import { useUserStore } from "@/stores/user";
import { fetchCurrentUser } from "@/api/account";
import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("@/views/Home.vue"),
    },
    {
      path: "/username",
      name: "userhome",
      component: () => import("@/views/UserHome.vue"),
    },
    {
      path: "/article/:id",
      name: "ArticleDetail",
      component: () => import("@/views/ArticleDetail.vue"),
    },
    {
      path: "/login",
      name: "Login",
      component: () => import("@/views/Login.vue"),
    },
    {
      path: "/admin",
      name: "Admin",
      meta: { requiresAdmin: true },
      component: () => import("@/views/Admin.vue"),
    },
    {
      path: "/admin/createArticle",
      name: "AdminCreate",
      meta: { requiresAdmin: true },
      component: () => import("@/views/AdminCreate.vue"),
    },
    {
      path: "/admin/edit/:id",
      name: "AdminEdit",
      meta: { requiresAdmin: true },
      component: () => import("@/views/AdminEdit.vue"),
    },
  ],
});

// 管理页面以服务端当前用户信息为准，不能信任 localStorage 中的身份字段。
router.beforeEach(async (to) => {
  if (!to.meta.requiresAdmin) return true;

  const user = useUserStore();
  const loginRoute = {
    name: "Login",
    query: { redirect: to.fullPath },
  };

  if (!user.token) return loginRoute;

  try {
    const response = await fetchCurrentUser();
    user.updateCurrentUser(response.data);
    return response.data.identity === "admin" ? true : { name: "home" };
  } catch {
    user.logout();
    return loginRoute;
  }
});

export default router;
