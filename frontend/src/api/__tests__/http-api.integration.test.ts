import { createPinia, setActivePinia } from "pinia";
import { HttpResponse, http } from "msw";
import { setupServer } from "msw/node";
import {
    afterAll,
    afterEach,
    beforeAll,
    beforeEach,
    describe,
    expect,
    it,
    vi,
} from "vitest";

import { getCurrentUser, login, register } from "@/api/auth";
import {
    createArticle,
    deleteArticle,
    getArticle,
    getArticleSuggestions,
    getArticles,
    updateArticle,
    updateArticleStatus,
} from "@/api/articles";
import {
    createComment,
    deleteComment,
    getComments,
    setCommentLike,
} from "@/api/comments";
import { addFavorite, getFavorites, removeFavorite } from "@/api/favorites";
import {
    deleteAvatar,
    deleteUser,
    getUsers,
    updateProfile,
    updateUser,
    uploadAvatar,
} from "@/api/users";
import { API_BASE_URL, resolveApiUrl } from "@/config";
import { useUserStore } from "@/stores/user";
import client from "@/api/client";

const server = setupServer();
const endpoint = (path: string) =>
    new URL(path, `${API_BASE_URL || window.location.origin}/`).toString();

beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
beforeEach(() => {
    localStorage.clear();
    setActivePinia(createPinia());
});

describe("shared HTTP API client", () => {
    it("uses canonical auth endpoints and injects the bearer token", async () => {
        server.use(
            http.post(endpoint("/api/auth/register"), async ({ request }) => {
                expect(await request.json()).toEqual({
                    username: "new-reader",
                    password: "password",
                });
                return HttpResponse.json(
                    {
                        token: "registration-token",
                        user_id: "user-2",
                        username: "new-reader",
                        identity: "user",
                    },
                    { status: 201 }
                );
            }),
            http.post(endpoint("/api/auth/login"), async ({ request }) => {
                expect(await request.json()).toEqual({
                    username: "reader",
                    password: "password",
                });
                return HttpResponse.json({
                    token: "token-1",
                    user_id: "user-1",
                    username: "reader",
                    identity: "user",
                });
            }),
            http.get(endpoint("/api/auth/me"), ({ request }) => {
                expect(request.headers.get("authorization")).toBe("Bearer token-1");
                return HttpResponse.json({
                    id: "user-1",
                    username: "reader",
                    identity: "user",
                    signature: "hello",
                    avatar_url: "/api/users/user-1/avatar",
                });
            })
        );

        const registration = await register({
            username: "new-reader",
            password: "password",
        });
        expect(registration.status).toBe(201);
        const auth = await login({ username: "reader", password: "password" });
        useUserStore().$patch({ token: auth.data.token });
        const current = await getCurrentUser();

        expect(current.data.signature).toBe("hello");
    });

    it("uses one article and search contract", async () => {
        server.use(
            http.get(endpoint("/api/articles"), ({ request }) => {
                expect(new URL(request.url).searchParams.get("query")).toBe("vue");
                return HttpResponse.json({ articles: [] });
            }),
            http.post(endpoint("/api/articles"), async ({ request }) =>
                HttpResponse.json(
                    { ...(await request.json() as object), id: "article-1", status: "draft" },
                    { status: 201 }
                )
            ),
            http.get(endpoint("/api/articles/article-1"), () =>
                HttpResponse.json({ id: "article-1", title: "Vue" })
            ),
            http.put(
                endpoint("/api/articles/article-1"),
                async ({ request }) => {
                    expect(await request.json()).toEqual({ title: "Updated Vue" });
                    return HttpResponse.json({
                        id: "article-1",
                        title: "Updated Vue",
                        status: "draft",
                    });
                }
            ),
            http.delete(
                endpoint("/api/articles/article-1"),
                () => new HttpResponse(null, { status: 204 })
            ),
            http.patch(
                endpoint("/api/articles/article-1/status"),
                async ({ request }) => {
                    expect(await request.json()).toEqual({ status: "published" });
                    return HttpResponse.json({ id: "article-1", status: "published" });
                }
            ),
            http.get(endpoint("/api/search/suggestions"), ({ request }) => {
                expect(new URL(request.url).searchParams.get("query")).toBe("vue");
                return HttpResponse.json({
                    items: [{ id: "article-1", title: "Vue" }],
                });
            })
        );

        await getArticles("vue");
        const created = await createArticle({ title: "Vue" });
        expect(created.data.id).toBe("article-1");
        expect((await getArticle("article-1")).data.title).toBe("Vue");
        expect(
            (await updateArticle("article-1", { title: "Updated Vue" })).data
                .title
        ).toBe("Updated Vue");
        await updateArticleStatus("article-1", "published");
        const suggestions = await getArticleSuggestions("vue");
        expect(suggestions.data.items).toHaveLength(1);
        expect((await deleteArticle("article-1")).status).toBe(204);
    });

    it("sends idempotent comment-like state", async () => {
        server.use(
            http.post(
                endpoint("/api/articles/article-1/comments"),
                async ({ request }) => {
                    expect(await request.json()).toEqual({
                        content: "hello",
                        parent_id: undefined,
                    });
                    return HttpResponse.json({ comment_id: "comment-1" }, { status: 201 });
                }
            ),
            http.put(
                endpoint("/api/comments/comment-1/like"),
                async ({ request }) => {
                    expect(await request.json()).toEqual({ liked: true });
                    return HttpResponse.json({
                        comment_id: "comment-1",
                        liked_by_me: true,
                        like_count: 1,
                    });
                }
            ),
            http.get(endpoint("/api/articles/article-1/comments"), () =>
                HttpResponse.json({
                    comments: [
                        {
                            comment_id: "comment-1",
                            like_count: 1,
                            liked_by_me: 1,
                        },
                    ],
                })
            ),
            http.delete(
                endpoint("/api/comments/comment-1"),
                () => new HttpResponse(null, { status: 204 })
            )
        );

        await createComment("article-1", "hello");
        expect((await getComments("article-1")).data.comments).toHaveLength(1);
        const result = await setCommentLike("comment-1", true);
        expect(result.data.like_count).toBe(1);
        expect((await deleteComment("comment-1")).status).toBe(204);
    });

    it("uses user-scoped favorite endpoints", async () => {
        server.use(
            http.get(endpoint("/api/favorites"), () =>
                HttpResponse.json({ articles: [{ id: "article-1" }] })
            ),
            http.put(endpoint("/api/favorites/article-1"), () =>
                HttpResponse.json({ article_id: "article-1", favorited: true })
            ),
            http.delete(endpoint("/api/favorites/article-1"), () =>
                HttpResponse.json({ article_id: "article-1", favorited: false })
            )
        );

        expect((await getFavorites()).data.articles).toHaveLength(1);
        expect((await addFavorite("article-1")).data.favorited).toBe(true);
        expect((await removeFavorite("article-1")).data.favorited).toBe(false);
    });

    it("uses canonical profile, admin-user and avatar endpoints", async () => {
        server.use(
            http.get(endpoint("/api/users"), ({ request }) => {
                expect(new URL(request.url).searchParams.get("limit")).toBe("25");
                return HttpResponse.json({
                    users: [
                        {
                            id: "user-1",
                            username: "reader",
                            identity: "user",
                        },
                    ],
                });
            }),
            http.patch(endpoint("/api/users/user-1"), async ({ request }) => {
                expect(await request.json()).toEqual({
                    username: "renamed",
                    identity: "user",
                });
                return new HttpResponse(null, { status: 204 });
            }),
            http.patch(endpoint("/api/users/me/profile"), async ({ request }) => {
                expect(await request.json()).toEqual({ signature: "hello" });
                return HttpResponse.json({
                    id: "user-1",
                    username: "reader",
                    identity: "user",
                    signature: "hello",
                    avatar_url: null,
                });
            }),
            http.delete(
                endpoint("/api/users/user-1"),
                () => new HttpResponse(null, { status: 204 })
            ),
            http.delete(endpoint("/api/users/me/avatar"), () =>
                HttpResponse.json({
                    id: "user-1",
                    username: "reader",
                    identity: "user",
                    signature: "hello",
                    avatar_url: null,
                })
            )
        );

        expect((await getUsers(25)).data.users).toHaveLength(1);
        await updateUser({
            edited_id: "user-1",
            edited_username: "renamed",
            edited_identity: "user",
        });
        expect((await updateProfile({ signature: "hello" })).data.signature).toBe(
            "hello"
        );
        expect((await deleteAvatar()).data.avatar_url).toBeNull();
        expect((await deleteUser("user-1")).status).toBe(204);
    });

    it("wraps avatar files in the multipart form field", async () => {
        const request = vi.spyOn(client, "put").mockResolvedValue({
            data: { avatar_url: "/api/users/user-1/avatar" },
        } as never);
        const avatar = new File(["image"], "avatar.png", { type: "image/png" });

        await uploadAvatar(avatar);

        expect(request).toHaveBeenCalledOnce();
        expect(request.mock.calls[0][0]).toBe("/api/users/me/avatar");
        const form = request.mock.calls[0][1] as FormData;
        expect(form.get("avatar")).toBe(avatar);
    });
});

describe("API URL resolution", () => {
    it("keeps absolute and data URLs unchanged", () => {
        expect(resolveApiUrl("https://cdn.example/avatar.png")).toBe(
            "https://cdn.example/avatar.png"
        );
        expect(resolveApiUrl("data:image/png;base64,AA==")).toBe(
            "data:image/png;base64,AA=="
        );
    });

    it("resolves server-relative avatar paths", () => {
        expect(resolveApiUrl("/api/users/user-1/avatar")).toContain(
            "/api/users/user-1/avatar"
        );
    });
});
