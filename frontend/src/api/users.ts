import type {
    CurrentUser,
    EditUserData,
    UpdateProfileData,
    User,
} from "@/types/user";
import client from "./client";

export interface UserListResponse {
    users: User[];
}

export const getUsers = (limit = 10) =>
    client.get<UserListResponse>("/api/users", { params: { limit } });

export const deleteUser = (userId: string) =>
    client.delete(`/api/users/${encodeURIComponent(userId)}`);

export const updateUser = (input: EditUserData) =>
    client.patch(`/api/users/${encodeURIComponent(input.edited_id)}`, {
        username: input.edited_username || undefined,
        password: input.edited_password || undefined,
        identity: input.edited_identity || undefined,
    });

export const updateProfile = (input: UpdateProfileData) =>
    client.patch<CurrentUser>("/api/users/me/profile", input);

export const uploadAvatar = (file: File) => {
    const form = new FormData();
    form.append("avatar", file);
    return client.put<CurrentUser>("/api/users/me/avatar", form);
};

export const deleteAvatar = () =>
    client.delete<CurrentUser>("/api/users/me/avatar");
