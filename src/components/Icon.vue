<!--
  统一 SVG 图标组件（基于 lucide-vue-next）
  用法：<Icon name="home" :size="20" />
  所有图标名使用 kebab-case
-->
<script setup lang="ts">
import {
  AlarmClock,
  ArrowLeft,
  BookOpen,
  Bot,
  Brain,
  Camera,
  ChartColumn,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  CirclePlus,
  CircleX,
  ClipboardList,
  Download,
  FileText,
  FlaskConical,
  Folder,
  House,
  Info,
  Link,
  MessageSquare,
  Palette,
  Pencil,
  Plus,
  RefreshCw,
  RotateCw,
  Search,
  Settings,
  Share2,
  Sparkles,
  SquarePen,
  Star,
  Target,
  Trash2,
  TriangleAlert,
  Trophy,
  Undo,
  Upload,
  User,
  Users,
  Wrench,
  X,
  ZoomIn,
  ZoomOut
} from '@lucide/vue'
import { type Component, computed } from 'vue'

const iconMap: Record<string, Component> = {
  house: House,
  plus: Plus,
  'clipboard-list': ClipboardList,
  'book-open': BookOpen,
  user: User,
  search: Search,
  settings: Settings,
  palette: Palette,
  bot: Bot,
  'flask-conical': FlaskConical,
  pencil: Pencil,
  'trash-2': Trash2,
  'file-text': FileText,
  info: Info,
  'circle-check': CircleCheck,
  'triangle-alert': TriangleAlert,
  'circle-x': CircleX,
  wrench: Wrench,
  camera: Camera,
  x: X,
  'message-square': MessageSquare,
  link: Link,
  'chart-column': ChartColumn,
  'alarm-clock': AlarmClock,
  brain: Brain,
  sparkles: Sparkles,
  star: Star,
  trophy: Trophy,
  target: Target,
  folder: Folder,
  'square-pen': SquarePen,
  'arrow-left': ArrowLeft,
  'refresh-cw': RefreshCw,
  download: Download,
  upload: Upload,
  'share-2': Share2,
  undo: Undo,
  'chevron-down': ChevronDown,
  'circle-plus': CirclePlus,
  'zoom-in': ZoomIn,
  'zoom-out': ZoomOut,
  'rotate-cw': RotateCw,
  'chevron-left': ChevronLeft,
  'chevron-right': ChevronRight,
  'circle-alert': CircleAlert,
  users: Users
}

const props = withDefaults(
  defineProps<{
    name: string
    size?: number
    color?: string
    strokeWidth?: number
  }>(),
  {
    size: 20,
    strokeWidth: 2
  }
)

const iconComponent = computed(() => iconMap[props.name])
</script>

<template>
  <component
    :is="iconComponent"
    :size="size"
    :stroke-width="strokeWidth"
    :color="color"
    class="fp-icon"
  />
</template>

<style scoped>
.fp-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
</style>
