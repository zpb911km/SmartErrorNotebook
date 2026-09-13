<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import { listTags } from '../api'
import { useLatestRequest } from '../composables/useLatestRequest'
import type { TagDraft } from '../services/questionEditor'
import { showError } from '../utils/notification'

type SelectedTag = { name: string; color: string }
const props = defineProps<{ currentTags?: SelectedTag[] }>()
const emit = defineEmits<{ (e: 'select', tags: SelectedTag[]): void }>()
const tags = ref<TagDraft[]>([])
const drafts = ref<SelectedTag[]>([])
const showAdd = ref(false)
const name = ref('')
const color = ref('#7c3aed')
const loading = ref(false)
const begin = useLatestRequest()
const key = (tag: SelectedTag) => JSON.stringify([tag.name, tag.color])
const options = computed(() =>
  [
    ...new Map(
      [...tags.value, ...drafts.value, ...(props.currentTags || [])]
        .filter((t) => !t.name.startsWith('[已删除]'))
        .map((t) => [key(t), t])
    ).values()
  ].map((t) => ({ label: t.name, value: key(t), tag: t }))
)
const selected = computed(() => (props.currentTags || []).map(key))
function select(keys: string[]) {
  emit(
    'select',
    keys
      .map((k) => options.value.find((o) => o.value === k)!.tag)
      .map((t) => ({ name: t.name, color: t.color }))
  )
}
function add() {
  if (!name.value.trim()) return
  const tag = { name: name.value.trim(), color: color.value }
  drafts.value.push(tag)
  emit('select', [
    ...new Map(
      [...(props.currentTags || []), tag].map((t) => [key(t), t])
    ).values()
  ])
  showAdd.value = false
  name.value = ''
}
async function load() {
  const current = begin()
  loading.value = true
  try {
    const data = await listTags()
    if (current()) tags.value = data
  } catch (error) {
    if (current()) showError('错因加载失败', String(error))
  } finally {
    if (current()) loading.value = false
  }
}
onMounted(load)
</script>
<template>
  <div>
    <q-select
      outlined
      dense
      multiple
      use-chips
      emit-value
      map-options
      :model-value="selected"
      :options="options"
      :loading="loading"
      label="错因标签"
      @update:model-value="select"
      @popup-show="load"
    >
      <template #after-options>
        <q-item clickable @click="showAdd = true">
          <q-item-section avatar> <q-icon name="add" /> </q-item-section
          ><q-item-section>添加新错因</q-item-section>
        </q-item>
      </template>
      <template #no-option>
        <q-item>
          <q-item-section>暂无错因</q-item-section
          ><q-item-section side>
            <q-btn
              flat
              label="添加错因"
              color="primary"
              @click="showAdd = true"
            />
          </q-item-section>
        </q-item>
      </template>
    </q-select>
    <q-dialog v-model="showAdd">
      <q-card class="dialog-card">
        <q-card-section class="text-h6"> 添加新错因 </q-card-section
        ><q-card-section class="q-gutter-md">
          <q-input
            v-model="name"
            outlined
            autofocus
            label="错因名称"
            @keyup.enter="add"
          /><q-color
            v-model="color"
            no-header-tabs
            no-footer
            default-view="palette"
            aria-label="标记颜色"
          /> </q-card-section
        ><q-card-actions align="right">
          <q-btn v-close-popup flat label="取消" /><q-btn
            unelevated
            color="primary"
            label="添加"
            :disable="!name.trim()"
            @click="add"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>
