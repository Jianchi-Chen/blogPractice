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
                    <input
                        ref="avatarInput"
                        class="avatar-input"
                        type="file"
                        accept="image/png,image/jpeg"
                        @change="handleAvatarSelected"
                    />
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
import { deleteAvatar, updateProfile, uploadAvatar } from "@/api/users";
import { useMessage } from "naive-ui";
import { ref, computed, watch } from "vue";
import { useUserStore } from "@/stores/user";

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
const avatarInput = ref<HTMLInputElement | null>(null);

const avatar = computed(() => userStore.avatarUrl);

const initials = computed(() => {
    const parts = props.user.name.split(" ");
    if (parts.length >= 2) {
        return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return props.user.name.slice(0, 2).toUpperCase();
});

const signature = computed(() => userStore.signature);
const draftSignature = ref("");
const editingSignature = ref(false);

watch(
    () => props.user.id,
    () => {
        draftSignature.value = userStore.signature || "";
    },
    { immediate: true }
);

// 头像选择和上传
const selectAvatar = async () => {
    avatarInput.value?.click();
};

const handleAvatarSelected = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
        const response = await uploadAvatar(file);
        userStore.updateCurrentUser(response.data);
        message.success("头像上传成功");
    } catch (error) {
        console.error("Upload avatar error:", error);
        message.error("头像上传失败");
    } finally {
        input.value = "";
    }
};

const removeAvatar = async () => {
    try {
        const response = await deleteAvatar();
        userStore.updateCurrentUser(response.data);
        message.success("头像已移除");
    } catch (error) {
        console.error("Remove avatar error:", error);
        message.error("头像移除失败");
    }
};

// 签名编辑
const saveSignature = async () => {
    try {
        const response = await updateProfile({
            signature: draftSignature.value.trim(),
        });
        userStore.updateCurrentUser(response.data);
        editingSignature.value = false;
        message.success("签名已保存");
    } catch (error) {
        console.error("Save signature error:", error);
        message.error("签名保存失败");
    }
};

const cancelSignature = () => {
    draftSignature.value = signature.value || "";
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

.avatar-input {
    display: none;
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
