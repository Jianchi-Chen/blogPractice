import { useUserStore } from "@/stores/user";
import { getCurrentUser } from "@/api/auth";
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
      path: "/profile",
      name: "Profile",
      meta: { requiresAuth: true },
      component: () => import("@/views/ProfileView.vue"),
    },
    {
      path: "/username",
      redirect: "/profile",
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
  if (!to.meta.requiresAdmin && !to.meta.requiresAuth) return true;

  const user = useUserStore();
  const loginRoute = {
    name: "Login",
    query: { redirect: to.fullPath },
  };

  if (!user.token) return loginRoute;

  try {
    const response = await getCurrentUser();
    user.updateCurrentUser(response.data);
    if (to.meta.requiresAdmin && response.data.identity !== "admin") {
      return { name: "home" };
    }
    return true;
  } catch {
    user.logout();
    return loginRoute;
  }
});

export default router;
