import type { AuthResponse, CurrentUser } from "@/types/user";
import client from "./client";

export interface RegisterInput {
    username: string;
    password: string;
    identity?: string;
}

export interface LoginInput {
    username: string;
    password: string;
}

export const register = (input: RegisterInput) =>
    client.post<AuthResponse>("/api/auth/register", input);

export const login = (input: LoginInput) =>
    client.post<AuthResponse>("/api/auth/login", input);

export const getCurrentUser = () => client.get<CurrentUser>("/api/auth/me");
