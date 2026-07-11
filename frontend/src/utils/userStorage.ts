export type UserStorageItem = "avatar" | "signature" | "favorites";

export const userStorageKey = (userId: string, item: UserStorageItem) =>
    `myblog:user:${encodeURIComponent(userId || "anonymous")}:${item}`;
