import z from "zod";

export const UserSchema = z.object({
    id: z.string().optional(),
    username: z.string().min(1, "不能为空"),
    password: z.string().min(1, "不能为空"),
    token: z.string().optional(),
    identity: z.string().default("visitor").optional(),
    avatarUrl: z.string().optional(),
    signature: z.string().optional(),
});

export type User = z.infer<typeof UserSchema>;

export interface AuthResponse {
    token: string;
    user_id: string;
    username: string;
    identity: string;
}

export interface CurrentUser {
    id: string;
    username: string;
    identity: string;
    signature: string;
    avatar_url: string | null;
}

export const createEmptyUser = (): User => {
    return UserSchema.parse({
        username: "unknown username",
        password: "unknow password",
    });
};

// 定义修改用户信息时的数据类型，不使用zod
export interface EditUserData {
    edited_id: string;
    edited_username: string;
    edited_password?: string;
    edited_identity: string;
}

export interface UpdateProfileData {
    signature: string;
}
