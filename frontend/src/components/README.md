# Components 组织结构

本项目的 Vue 组件按功能模块进行了分类组织，以提高代码的可维护性和可读性。

## 📁 目录结构

```
components/
├── admin/          # 管理员相关组件
├── article/        # 文章相关组件
├── user/           # 用户相关组件
├── layout/         # 布局相关组件
├── common/         # 通用/共享组件
└── icons/          # 图标组件
```

## 📦 组件分类

### 🔧 admin/ - 管理员功能
- **EditUserDialog.vue** - 编辑用户对话框
- **NewUserDialog.vue** - 新建用户对话框
- **ArticleAction.vue** - 文章操作组件（编辑、删除等）
- **StatusTag.vue** - 文章状态标签

**使用场景**: Admin.vue

### 📝 article/ - 文章内容
- **ArticleForm.vue** - 文章表单（创建/编辑）
- **CommentSection.vue** - 评论区组件
- **EntryCommentBar.vue** - 评论输入栏
- **MdPreview.vue** - Markdown 预览组件

**使用场景**: AdminCreate.vue, AdminEdit.vue, ArticleDetail.vue

### 👤 user/ - 用户资料
- **UserProfile.vue** - 用户资料卡片（头像、签名）
- **FavoriteArticles.vue** - 用户收藏文章列表

**使用场景**: ProfileView.vue

### 🎨 layout/ - 页面布局
- **NavBar.vue** - 顶部导航栏
- **NavSearch.vue** - 导航搜索组件
- **Sider.vue** - 侧边栏

**使用场景**: App.vue, 全局布局

### 🔄 common/ - 通用组件
- **MarkdownEditor.vue** - Markdown 编辑器（Vditor）
- **Welcome.vue** - 欢迎页组件

**使用场景**: 多个页面共享使用

## 📖 使用方式

### 方式一：直接导入（推荐）
```typescript
import UserProfile from '@/components/user/UserProfile.vue';
import ArticleForm from '@/components/article/ArticleForm.vue';
```

### 方式二：通过 index 批量导入
```typescript
import { UserProfile, FavoriteArticles } from '@/components/user';
import { ArticleForm, CommentSection } from '@/components/article';
```

## 🎯 命名规范

- 组件文件使用 **PascalCase** 命名（如 `UserProfile.vue`）
- 文件夹使用 **小写** 命名（如 `admin/`, `article/`）
- 每个文件夹包含 `index.ts` 用于批量导出

## 🔍 查找组件

如果不确定某个组件在哪个文件夹，可以参考以下规则：
- 只在管理页面使用？→ `admin/`
- 与文章内容相关？→ `article/`
- 用户资料相关？→ `user/`
- 导航/布局相关？→ `layout/`
- 多处共享使用？→ `common/`
