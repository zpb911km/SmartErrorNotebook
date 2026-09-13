<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { createSubject, listSubjects } from '../api'
import { useLatestRequest } from '../composables/useLatestRequest'
import type { Subject } from '../types'
import { showError } from '../utils/notification'

const props = defineProps<{ modelValue: string; disabled?: boolean }>()
const emit = defineEmits<{
  (e: 'select', subjectId: string): void
  (e: 'update:modelValue', subjectId: string): void
}>()
const subjects = ref<Subject[]>([])
const showAdd = ref(false)
const saving = ref(false)
const loading = ref(false)
const loadError = ref(false)
const name = ref('')
const color = ref('#2563eb')
const begin = useLatestRequest()
function select(value: string | null) {
  emit('select', value || '')
  emit('update:modelValue', value || '')
}
async function load() {
  const current = begin()
  loading.value = true
  loadError.value = false
  try {
    const data = await listSubjects()
    if (current()) subjects.value = data
  } catch {
    if (current()) loadError.value = true
  } finally {
    if (current()) loading.value = false
  }
}
async function add() {
  if (!name.value.trim() || saving.value) return
  saving.value = true
  try {
    const result = await createSubject({
      name: name.value.trim(),
      color: color.value
    })
    subjects.value.push(result)
    select(result.id)
    showAdd.value = false
    name.value = ''
  } catch (error) {
    showError('添加科目失败', String(error))
  } finally {
    saving.value = false
  }
}
onMounted(load)
</script>
<template>
  <div>
    <q-select
      outlined
      dense
      clearable
      :model-value="props.modelValue || null"
      :options="subjects"
      option-value="id"
      option-label="name"
      emit-value
      map-options
      :disable="props.disabled"
      :loading="loading"
      label="科目"
      @update:model-value="select"
      @popup-show="load"
    >
      <template #after-options>
        <q-item clickable :disable="props.disabled" @click="showAdd = true">
          <q-item-section avatar> <q-icon name="add" /> </q-item-section
          ><q-item-section>添加新科目</q-item-section>
        </q-item>
      </template>
      <template #no-option>
        <q-item>
          <q-item-section>暂无科目</q-item-section
          ><q-item-section side>
            <q-btn
              flat
              color="primary"
              label="添加科目"
              @click="showAdd = true"
            />
          </q-item-section>
        </q-item>
      </template>
    </q-select>
    <q-banner v-if="loadError" dense class="error-banner q-mt-sm">
      科目加载失败<template #action>
        <q-btn flat label="重试" @click="load" />
      </template>
    </q-banner>
    <q-dialog v-model="showAdd" :persistent="saving">
      <q-card class="dialog-card">
        <q-card-section class="text-h6"> 添加新科目 </q-card-section
        ><q-card-section class="q-gutter-md">
          <q-input
            v-model="name"
            outlined
            autofocus
            label="科目名称"
            @keyup.enter="add"
          /><q-color
            v-model="color"
            no-header-tabs
            no-footer
            default-view="palette"
            aria-label="标记颜色"
          /> </q-card-section
        ><q-card-actions align="right">
          <q-btn v-close-popup flat label="取消" :disable="saving" /><q-btn
            unelevated
            color="primary"
            label="添加"
            :loading="saving"
            :disable="!name.trim()"
            @click="add"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>
