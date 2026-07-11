<template>
    <n-data-table
        :columns="columns"
        :data="users"
        :pagination="pagination"
        :bordered="true"
        ref="tableRef"
        @update:sorter="handleSorterChange"
    />
</template>

<script setup lang="ts">
import { NDataTable, NButton, NTag, useDialog, useMessage } from "naive-ui";
import type { DataTableBaseColumn, DataTableSortState } from "naive-ui";
import { deleteUser as deleteUserApi } from "@/api/users";
import { h, ref } from "vue";
import type { User } from "@/types/user";

interface UserRowData {
    id: string;
    key: string;
    username: string;
    identity: string;
    created_at: string;
}

const props = defineProps<{
    users: User[];
}>();

const emit = defineEmits<{
    refresh: [];
    edit: [user: User];
}>();

const dialog = useDialog();
const message = useMessage();
const tableRef = ref();

// 用户管理表格列
const columns = ref<DataTableBaseColumn<UserRowData>[]>([
    {
        title: "用户名",
        key: "username",
    },
    {
        title: "身份",
        key: "identity",
        sortOrder: "descend",
        sorter(rowA, rowB) {
            const identityOrder: Record<string, number> = {
                admin: 3,
                user: 2,
                visitor: 1,
            };
            return identityOrder[rowA.identity] - identityOrder[rowB.identity];
        },
        render(row) {
            return h(
                NTag,
                {
                    type:
                        row.identity === "admin"
                            ? "error"
                            : row.identity === "user"
                            ? "success"
                            : "warning",
                },
                {
                    default: () => row.identity,
                }
            );
        },
    },
    {
        title: "操作",
        key: "actions",
        render(row) {
            return h("div", {}, [
                h(
                    NButton,
                    {
                        type: "primary",
                        size: "small",
                        onClick: () => {
                            const user = props.users.find((u) => u.id === row.id);
                            if (user) {
                                emit("edit", user);
                            }
                        },
                    },
                    { default: () => "编辑" }
                ),
                h(
                    NButton,
                    {
                        type: "error",
                        size: "small",
                        style: { "margin-left": "8px" },
                        onClick: () => {
                            handleDelete(row.id);
                        },
                    },
                    { default: () => "删除" }
                ),
            ]);
        },
    },
]);

const pagination = {
    pageSize: 50,
};

// 删除用户
const handleDelete = async (id: User["id"]) => {
    dialog.warning({
        title: "确认删除?",
        content: "此操作将永久删除该用户，是否继续?",
        positiveText: "确定",
        negativeText: "取消",
        onPositiveClick: async () => {
            if (id == null) {
                message.error("无效的用户ID, 无法删除");
                return;
            }
            try {
                await deleteUserApi(id);
                message.success("用户删除成功");
                emit("refresh");
            } catch (error) {
                message.error("请求出错，无法删除用户");
            }
        },
    });
};

// 处理排序
const handleSorterChange = (sorter: DataTableSortState) => {
    columns.value.forEach((column) => {
        if (column.sortOrder === undefined) return;
        if (!sorter) {
            column.sortOrder = false;
            return;
        }
        if (column.key === sorter.columnKey) {
            column.sortOrder = sorter.order;
        } else {
            column.sortOrder = false;
        }
    });
};

// 清空排序
const clearSorter = () => {
    tableRef.value?.filter(null);
    tableRef.value?.sort(null);
};

defineExpose({ clearSorter });
</script>
