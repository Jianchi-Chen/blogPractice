<template>
    <n-modal
        preset="dialog"
        :title="'创建新用户'"
        :show="show"
        @update:show="(val: boolean) => emit('update:show', val)"
    >
        <n-flex>
            <n-form ref="formRef" :model="modelRef" :rules="rules">
                <n-form-item-row label="用户名" path="username">
                    <n-input
                        v-model:value="modelRef.username"
                        :placeholder="'请输入用户名'"
                    />
                </n-form-item-row>

                <n-form-item-row label="密码" path="password">
                    <n-input
                        type="password"
                        v-model:value="modelRef.password"
                        :placeholder="'请输入密码'"
                    />
                </n-form-item-row>

                <n-form-item-row label="重复密码" path="reenteredPassword">
                    <n-input
                        type="password"
                        v-model:value="modelRef.reenteredPassword"
                        :placeholder="'请再次输入密码'"
                    />
                </n-form-item-row>
            </n-form>

            <n-flex horizontal align="center">
                <n-text>选择用户身份:</n-text>
                <n-space vertical>
                    <n-radio-group
                        v-model:value="radio_button_value"
                        name="radiobuttongroup1"
                    >
                        <n-radio-button
                            v-for="identity in identities"
                            :key="identity.value"
                            :value="identity.value"
                            :label="identity.label"
                            :disabled="identity.disabled"
                        />
                    </n-radio-group>
                </n-space>
            </n-flex>

            <n-button type="primary" :loading="loading" @click="createNewUser">
                创建用户
            </n-button>
            <n-button type="error" :loading="loading" @click="cleanForm">
                清空表单
            </n-button>
        </n-flex>
    </n-modal>
</template>

<script setup lang="ts">
import { register } from "@/api/auth";
import type { User } from "@/types/user";
import {
    useMessage,
    type FormInst,
    type FormItemRule,
    type FormRules,
} from "naive-ui";
import { computed, ref, watch, type Ref } from "vue";
import { boolean } from "zod";
import { de } from "zod/locales";

interface ModelType {
    username: string;
    password: string;
    reenteredPassword: string;
}

const modelRef = ref<ModelType>({
    username: "",
    password: "",
    reenteredPassword: "",
});

const loading = ref(false);
const formRef = ref<FormInst | null>(null);
const message = useMessage();
const radio_button_value: Ref<User["identity"] | null> = ref("user");

const props = defineProps<{
    show: boolean;
}>();

const emit = defineEmits<{
    "update:show": [boolean];
    success: [];
}>();

const identities = [
    {
        label: "管理员",
        value: "admin",
        disabled: true,
    },
    {
        label: "普通用户",
        value: "user",
        default: true,
    },
    {
        label: "游客",
        value: "visitor",
    },
];

const rules = {
    username: [
        {
            required: true,
            message: "请输入用户名",
            trigger: ["input", "blur"],
        },
        {
            min: 2,
            max: 20,
            message: "用户名长度 2-20",
            trigger: ["blur", "input"],
        },
    ],
    password: [
        {
            required: true,
            message: "请输入密码",
            trigger: ["blur", "input"],
        },
        {
            min: 6,
            max: 20,
            message: "密码长度 6-20",
            trigger: ["blur", "input"],
        },
    ],
    reenteredPassword: [
        {
            required: true,
            message: "请再次输入密码",
            trigger: ["input", "blur"],
        },
        {
            // 验证密码逻辑
            validator: (rule: any, value: string) => {
                return modelRef.value.password === value
                    ? true
                    : new Error("两次密码输入不一致");
            },
            message: "两次密码输入不一致",
            trigger: "blur",
        },
    ],
};
// 创建新用户
const createNewUser = async () => {
    loading.value = true;
    try {
        await formRef.value?.validate();
        const datamodel: {
            username: string;
            password: string;
            identity?: string;
        } = {
            username: modelRef.value.username,
            password: modelRef.value.password,
            identity: radio_button_value.value || "user",
        };
        await register(datamodel);
        message.success("创建成功");
        emit("success");
    } catch (err) {
        console.error(err);
        message.error("注册失败");
    } finally {
        loading.value = false;
    }
};

// 清空表单
const cleanForm = () => {
    modelRef.value.username = "";
    modelRef.value.password = "";
    modelRef.value.reenteredPassword = "";
};
</script>
