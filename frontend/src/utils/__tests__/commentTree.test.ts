import { describe, expect, it } from "vitest";

import type { ApiComment } from "@/types/comment";
import { buildCommentTree, findCommentNode } from "@/utils/commentTree";

const comment = (
    commentId: string,
    parentId: string | null = null,
    overrides: Partial<ApiComment> = {}
): ApiComment => ({
    comment_id: commentId,
    parent_id: parentId,
    like_count: 0,
    liked_by_me: 0,
    ...overrides,
});

describe("comment tree", () => {
    it("builds parent-child relationships regardless of input order", () => {
        const roots = buildCommentTree([
            comment("reply", "parent"),
            comment("parent"),
        ]);

        expect(roots.map((item) => item.comment_id)).toEqual(["parent"]);
        expect(roots[0].children.map((item) => item.comment_id)).toEqual([
            "reply",
        ]);
        expect(findCommentNode(roots, "reply")?.parent_id).toBe("parent");
    });

    it("normalizes nullable counters and keeps orphan replies visible", () => {
        const roots = buildCommentTree([
            comment("orphan", "missing", {
                like_count: null,
                liked_by_me: null,
            }),
        ]);

        expect(roots).toHaveLength(1);
        expect(roots[0]).toMatchObject({
            comment_id: "orphan",
            like_count: 0,
            liked_by_me: 0,
        });
    });
});
