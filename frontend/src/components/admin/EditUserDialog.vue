<template>
    <n-modal
        preset="dialog"
        :title="'编辑用户'"
        :show="show"
        @update:show="(val: boolean) => emit('update:show', val)"
    >
        <n-flex>
            <n-form
                ref="formRef"
                :model="modelRef"
                :rules="rules"
                label-placement="left"
                label-width="auto"
            >
                <n-form-item label="用户名" path="username">
                    <n-input
                        v-model:value="modelRef.username"
                        :placeholder="'编辑用户名'"
                    />
                </n-form-item>

                <n-form-item label="密码" path="password">
                    <n-input
                        type="password"
                        v-model:value="modelRef.password"
                        :placeholder="'编辑新密码，如果需要的话'"
                    />
                </n-form-item>

                <n-form-item label="重复密码" path="reenteredPassword">
                    <n-input
                        type="password"
                        v-model:value="modelRef.reenteredPassword"
                        :placeholder="'请再次输入新密码'"
                    />
                </n-form-item>
            </n-form>

            <n-flex horizontal align="center">
                <n-text>设置用户身份:</n-text>
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
                        />
                    </n-radio-group>
                </n-space>

                <n-button type="primary" :loading="loading" @click="EditUser">
                    编辑用户
                </n-button>
                <n-button type="error" :loading="loading" @click="cleanForm">
                    清空表单
                </n-button>
            </n-flex>
        </n-flex>
    </n-modal>
</template>

<script setup lang="ts">
import { updateUser } from "@/api/users";
import type { EditUserData, User } from "@/types/user";
import {
    useMessage,
    type FormInst,
    type FormItemRule,
    type FormRules,
} from "naive-ui";
import { computed, ref, watch, type Ref } from "vue";
import { boolean } from "zod";
import { de } from "zod/locales";

const props = defineProps<{
    show: boolean;
    userdata: User | null;
}>();
const emit = defineEmits<{
    "update:show": [boolean];
    success: [];
}>();

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
const revisedPassword = ref("false");
const formRef = ref<FormInst | null>(null);
const message = useMessage();
const radio_button_value: Ref<User["identity"] | null> = ref(
    props.userdata ? props.userdata.identity : "user"
);

const identities = [
    {
        label: "管理员",
        value: "admin",
    },
    {
        label: "普通用户",
        value: "user",
    },
    {
        label: "游客",
        value: "visitor",
    },
];

const rules = {
    username: [
        {
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

// 编辑用户
const EditUser = async () => {
    loading.value = true;
    try {
        await formRef.value?.validate();

        let payload: EditUserData = {
            edited_id: props.userdata?.id ? props.userdata.id : "",
            edited_username: modelRef.value.username,
            edited_password: modelRef.value.password
                ? modelRef.value.password
                : "",
            edited_identity: radio_button_value.value || "user",
        };
        console.log(payload);
        await updateUser(payload);
        message.success("编辑成功");
        emit("success");
    } catch (err) {
        console.error("edit failed" + err);
        message.error("编辑失败");
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

// 监听 userdata 的变化，更新表单数据
watch(
    () => props.userdata,
    (newUserdata) => {
        if (newUserdata) {
            modelRef.value.username = newUserdata.username;
            radio_button_value.value = newUserdata.identity;
        } else {
            cleanForm();
            radio_button_value.value = "user";
        }
    },
    { immediate: true }
);
</script>
