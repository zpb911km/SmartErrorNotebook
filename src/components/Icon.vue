<!--
  统一 SVG 图标组件（基于 lucide-vue-next）
  用法：<Icon name="home" :size="20" />
  所有图标名使用 kebab-case
-->
<script setup lang="ts">
import { computed } from 'vue'
import {
  House,
  Plus,
  ClipboardList,
  BookOpen,
  User,
  Search,
  Settings,
  Palette,
  Bot,
  FlaskConical,
  Pencil,
  Trash2,
  FileText,
  Info,
  CircleCheck,
  TriangleAlert,
  CircleX,
  Wrench,
  Camera,
  X,
  MessageSquare,
  Link,
  ChartColumn,
  AlarmClock,
  Brain,
  Sparkles,
  Star,
  Trophy,
  Target,
  Folder,
  SquarePen,
  ArrowLeft,
  RefreshCw,
  Download,
  Upload,
  Share2,
  Undo,
  ChevronDown,
  CirclePlus,
  ZoomIn,
  ZoomOut,
  RotateCw,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  Users
} from '@lucide/vue'

const iconMap: Record<string, any> = {
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
