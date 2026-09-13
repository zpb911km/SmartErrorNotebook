<script setup lang="ts">
import { ref } from 'vue'

const open = ref(false)
</script>

<template>
  <q-card v-if="$q.screen.width >= 1024" flat bordered>
    <div class="filter-panel">
      <slot />
    </div>
  </q-card>
  <template v-else>
    <q-btn
      outline
      color="primary"
      icon="tune"
      label="筛选与排序"
      @click="open = true"
    />
    <q-dialog v-model="open" position="right">
      <q-card class="mobile-filter-card">
        <div class="filter-panel-mobile">
          <div class="row items-center justify-between">
            <h2>筛选与排序</h2>
            <q-btn
              flat
              round
              icon="close"
              aria-label="关闭筛选"
              @click="open = false"
            />
          </div>
          <slot />
          <q-btn
            unelevated
            color="primary"
            label="查看结果"
            @click="open = false"
          />
        </div>
      </q-card>
    </q-dialog>
  </template>
</template>

<style scoped>
.mobile-filter-card {
  width: 340px;
  max-width: 100vw;
  min-height: 100%;
}
</style>
