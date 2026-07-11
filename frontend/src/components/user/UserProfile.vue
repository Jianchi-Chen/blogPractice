<template>
    <n-card class="profile-card" size="large" bordered>
        <!-- 头像 -->
        <div class="profile-top">
            <div class="avatar-wrap">
                <n-avatar
                    :size="96"
                    :src="avatar || undefined"
                    :text="initials"
                    @click="handleShowAvatar"
                    :lazy="true"
                    object-fit="cover"
                    style="cursor: pointer"
                />
                <div class="avatar-actions">
                    <n-button size="tiny" secondary @click="selectAvatar"
                        >上传头像</n-button
                    >
                    <n-button size="tiny" tertiary @click="removeAvatar"
                        >移除</n-button
                    >
                </div>
            </div>
            <div class="profile-info">
                <div class="username">{{ user.name }}</div>
                <div class="meta">ID: {{ user.id }}</div>
            </div>
        </div>

        <n-divider />

        <!-- 签名 -->
        <div class="signature-section">
            <div class="section-header">
                <div class="title">个性签名</div>
                <div>
                    <n-button
                        size="tiny"
                        @click="editingSignature = !editingSignature"
                    >
                        {{ editingSignature ? "取消" : "编辑" }}
                    </n-button>
                </div>
            </div>

            <div v-if="editingSignature" class="signature-edit">
                <n-input
                    type="textarea"
                    v-model:value="draftSignature"
                    :rows="3"
                    placeholder="写下你的个性签名..."
                    maxlength="120"
                    show-count
                />
                <div class="actions">
                    <n-button
                        size="small"
                        @click="saveSignature"
                        type="primary"
                        >保存</n-button
                    >
                    <n-button size="small" @click="cancelSignature"
                        >取消</n-button
                    >
                </div>
            </div>

            <div v-else class="signature-display">
                <div v-if="signature" class="signature-text">
                    "{{ signature }}"
                </div>
                <n-empty v-else description="还没有签名，去编辑一个吧" />
            </div>
        </div>
    </n-card>

    <!-- 头像大图查看模态框 -->
    <n-modal
        v-model:show="showAvatarModal"
        preset="card"
        title="用户头像"
        :style="{ width: '80%', maxWidth: '800px' }"
        :bordered="false"
    >
        <div class="avatar-preview">
            <img :src="avatar" alt="用户头像" />
        </div>
    </n-modal>
</template>

<script setup lang="ts">
import { uploadAvatar } from "@/api/account";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir, join } from "@tauri-apps/api/path";
import { readFile } from "@tauri-apps/plugin-fs";
import { useMessage } from "naive-ui";
import { ref, computed, watch } from "vue";
import { useUserStore } from "@/stores/user";
import { userStorageKey } from "@/utils/userStorage";

interface UserInfo {
    id: string;
    name: string;
}

const props = defineProps<{
    user: UserInfo;
}>();

const message = useMessage();
const userStore = useUserStore();
const showAvatarModal = ref(false);

const avatar = computed(() => userStore.avatarUrl);

const initials = computed(() => {
    const parts = props.user.name.split(" ");
    if (parts.length >= 2) {
        return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return props.user.name.slice(0, 2).toUpperCase();
});

const signatureKey = computed(() =>
    userStorageKey(props.user.id, "signature")
);
const signature = ref("");
const draftSignature = ref("");
const editingSignature = ref(false);

watch(
    () => props.user.id,
    () => {
        signature.value = localStorage.getItem(signatureKey.value) || "";
        draftSignature.value = signature.value;
    },
    { immediate: true }
);

// 头像选择和上传
const selectAvatar = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "图片",
                    extensions: ["png", "jpg", "jpeg"],
                },
            ],
        });

        if (!selected) {
            return;
        }

        const filePath = selected as string;
        const result = await uploadAvatar(filePath);

        const dataDir = await appDataDir();
        const fullPath = await join(dataDir, result.path);

        const fileBytes = await readFile(fullPath);
        const base64 = btoa(
            new Uint8Array(fileBytes).reduce(
                (data, byte) => data + String.fromCharCode(byte),
                ""
            )
        );

        const ext = result.path.split(".").pop()?.toLowerCase();
        const mimeType =
            ext === "png"
                ? "image/png"
                : ext === "jpg" || ext === "jpeg"
                ? "image/jpeg"
                : "image/jpeg";

        const base64Url = `data:${mimeType};base64,${base64}`;

        userStore.setAvatar(base64Url);

        message.success("头像上传成功！");
    } catch (error: any) {
        console.error("Upload avatar error:", error);
        message.error(error || "头像上传失败");
    }
};

const removeAvatar = () => {
    userStore.setAvatar("");
};

// 签名编辑
const saveSignature = () => {
    signature.value = draftSignature.value.trim();
    localStorage.setItem(signatureKey.value, signature.value);
    editingSignature.value = false;
};

const cancelSignature = () => {
    draftSignature.value = signature.value;
    editingSignature.value = false;
};

// 查看头像大图
const handleShowAvatar = () => {
    if (avatar.value) {
        showAvatarModal.value = true;
    } else {
        message.info("当前没有设置头像");
    }
};
</script>

<style scoped>
.profile-card {
    height: 100%;
}

.profile-top {
    display: flex;
    gap: 16px;
    align-items: center;
}

.avatar-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
}

.avatar-actions {
    display: flex;
    gap: 8px;
}

.profile-info .username {
    font-weight: 600;
    font-size: 18px;
}

.profile-info .meta {
    color: var(--n-typography-3);
    font-size: 12px;
    margin-top: 6px;
}

.signature-section .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.signature-edit .actions {
    margin-top: 8px;
    display: flex;
    gap: 8px;
}

.signature-display .signature-text {
    font-style: italic;
    color: var(--n-typography-2);
    padding: 8px 0;
}

.avatar-preview {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 40px 0;
}

.avatar-preview img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
}
</style>
