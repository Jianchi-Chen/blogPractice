<template>
    <n-data-table
        :columns="columns"
        :data="articles"
        :pagination="pagination"
        :bordered="true"
        ref="tableRef"
        @update:sorter="handleSorterChange"
    />
</template>

<script setup lang="ts">
import { NDataTable, NButton, useDialog, useMessage } from "naive-ui";
import type { DataTableBaseColumn, DataTableSortState } from "naive-ui";
import { deleteArticle, updateArticleStatus } from "@/api/articles";
import { computed, h, ref } from "vue";
import { useRouter } from "vue-router";
import StatusTag from "@/components/admin/StatusTag.vue";
import ArticleAction from "@/components/admin/ArticleAction.vue";
import type { Article } from "@/types/article";
import { useArticleStore } from "@/stores/article";

interface RowData {
    id: string;
    key: string;
    title: string;
    created_at: string;
    status: Article["status"];
    tags: string;
}

const props = defineProps<{
    articles: Article[];
}>();

const emit = defineEmits<{
    refresh: [];
}>();

const router = useRouter();
const dialog = useDialog();
const message = useMessage();
const tableRef = ref();
const articleStore = useArticleStore();

// 收集所有tag
const allTags = computed(() => {
    const set = new Set<string>();
    props.articles.forEach((i) => {
        if (i.tags) set.add(i.tags);
    });
    return Array.from(set);
});

// 标签筛选列
const titleColumn: DataTableBaseColumn<RowData> = {
    title: "标题",
    key: "title",
    filterOptions: computed(() => {
        return Array.from(allTags.value).map((tag) => ({
            label: tag,
            value: tag,
        }));
    }).value,
    filter(value, row) {
        return row.tags === value;
    },
};

// 时间排序列
const timeColumn: DataTableBaseColumn<RowData> = {
    title: "创建时间",
    key: "created_at",
    sortOrder: false,
    sorter(rowA, rowB) {
        return (
            new Date(rowA.created_at).getTime() -
            new Date(rowB.created_at).getTime()
        );
    },
};

// 状态排序列
const statusOrder = {
    draft: 1,
    archived: 2,
    published: 3,
};

const StatusColumn: DataTableBaseColumn<RowData> = {
    title: "Status",
    key: "status",
    sortOrder: false,
    render(row) {
        return h(StatusTag, { status: row.status });
    },
    sorter(rowA, rowB) {
        return statusOrder[rowA.status] - statusOrder[rowB.status];
    },
};

// 操作列
const ActionColumn: DataTableBaseColumn<RowData> = {
    title: "操作",
    key: "actions",
    render(row) {
        return h(ArticleAction, {
            id: row.id,
            status: row.status,
            onEdit: handleEdit,
            onToggleStatus: handleToggleStatus,
            onDelete: handleDelete,
        });
    },
};

const columns = ref<DataTableBaseColumn<RowData>[]>([
    titleColumn,
    timeColumn,
    StatusColumn,
    ActionColumn,
]);

const pagination = {
    pageSize: 50,
};

// 编辑文章
const handleEdit = (id: Article["id"]) => {
    router.push(`/admin/edit/${id}`);
};

// 删除文章
const handleDelete = async (id: Article["id"]) => {
    dialog.warning({
        title: "确认删除?",
        content: "此操作将永久删除该文章，是否继续?",
        positiveText: "确定",
        negativeText: "取消",
        onPositiveClick: async () => {
            await deleteArticle(id);
            message.success("Delete succeeded");
            emit("refresh");
        },
    });
};

// 转换文章状态
const handleToggleStatus = async (
    id: Article["id"],
    status: "draft" | "published" | "archived"
) => {
    await updateArticleStatus(id, status);
    articleStore.updateFolderContentSignal = true;
    emit("refresh");
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
