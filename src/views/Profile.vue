<template>
  <div class="profile-page">
    <div class="page-heading">
      <h1>看见自己的进步</h1>
      <p>从科目分布与复习表现，找到下一次学习的方向。</p>
    </div>
    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <q-spinner color="primary" size="28px" />
      <div>加载中...</div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="loadError" class="error-state">
      <Icon name="circle-alert" :size="48" class="error-icon" />
      <div class="error-description">
        {{ loadError }}
      </div>
      <q-btn
        no-caps
        unelevated
        type="button"
        flat
        class="retry-btn"
        @click="loadData"
      >
        重试
      </q-btn>
    </div>

    <!-- 正常内容 -->
    <template v-else>
      <!-- 概览卡片 -->
      <div class="overview-cards">
        <q-card flat bordered class="overview-card">
          <Icon name="chart-column" :size="24" class="card-icon" />
          <div class="card-content">
            <div class="card-value">
              {{ animatedTotal }}
            </div>
            <div class="card-label">总题目数</div>
          </div>
        </q-card>
        <q-card
          flat
          bordered
          tabindex="0"
          role="button"
          class="overview-card"
          style="cursor: pointer"
          @keydown.enter="$router.push('/review')"
          @keydown.space.prevent="$router.push('/review')"
          @click="$router.push('/review')"
        >
          <Icon name="alarm-clock" :size="24" class="card-icon" />
          <div class="card-content">
            <div class="card-value due">
              {{ animatedDue }}
            </div>
            <div class="card-label">待复习</div>
          </div>
        </q-card>
        <q-card flat bordered class="overview-card">
          <Icon name="brain" :size="24" class="card-icon" />
          <div class="card-content">
            <div class="card-value memory">
              {{ animatedMemory }}
            </div>
            <div class="card-label">目前记忆</div>
          </div>
        </q-card>
        <q-card flat bordered class="overview-card">
          <Icon name="sparkles" :size="24" class="card-icon" />
          <div class="card-content">
            <div class="card-value new">
              {{ animatedNewCards }}
            </div>
            <div class="card-label">新卡片数</div>
          </div>
        </q-card>
      </div>

      <!-- SRS 详细指标 -->
      <div class="stats-row">
        <div class="stat-item">
          <div class="stat-label">平均记忆强度</div>
          <div class="stat-value">
            {{ srsStats.averageStability.toFixed(1) }} 天
          </div>
        </div>
        <div class="stat-item">
          <div class="stat-label">平均难度</div>
          <div class="stat-value">
            {{ srsStats.averageDifficulty.toFixed(2) }}
          </div>
          <div class="stat-sub">范围 [1, 10]，越高越难</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">累计复习</div>
          <div class="stat-value">{{ srsStats.totalReviews }} 次</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">SRS 卡片</div>
          <div class="stat-value">
            {{ srsStats.total }}
          </div>
        </div>
      </div>

      <!-- 科目分布 -->
      <q-card flat bordered class="chart-section">
        <div class="section-header">
          <h3>科目分布</h3>
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="manage-btn"
            @click="openManageCascade"
          >
            <Icon name="settings" :size="18" class="manage-icon" />
            <span>管理</span>
          </q-btn>
        </div>

        <!-- 可视化科目分布条 -->
        <div v-if="subjectDistribution.length > 0" class="distribution-visual">
          <div class="stacked-bar">
            <div
              v-for="(item, index) in subjectDistribution"
              :key="index"
              class="bar-segment"
              :style="{
                width: item.percent + '%',
                background: item.color
              }"
              :class="{
                'segment-selected': selectedSubject?.name === item.name
              }"
              :title="`${item.name}: ${item.count}题 (${item.percent}%)`"
              @click="handleSelectSubjectByName(item.name)"
            />
          </div>
          <div class="bar-total">{{ overview.total }} 题</div>
          <div class="legend">
            <div
              v-for="(item, index) in subjectDistribution"
              :key="index"
              class="legend-item"
              :class="{
                'legend-selected': selectedSubject?.name === item.name
              }"
              @click="handleSelectSubjectByName(item.name)"
            >
              <span class="legend-color" :style="{ background: item.color }" />
              <span class="legend-label">{{ item.name }}</span>
              <span class="legend-value"
                >{{ item.count }}题 ({{ item.percent }}%)</span
              >
            </div>
          </div>
        </div>

        <div
          v-if="showCascade"
          ref="cascadeContainer"
          class="cascade-container"
        >
          <div class="cascade-header">
            <span class="cascade-title">科目详情</span>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="close-btn"
              @click="closeCascade"
            >
              <Icon name="x" :size="16" />
            </q-btn>
          </div>
          <div class="cascade-scroll-wrapper">
            <div class="cascade-column-wrapper">
              <!-- 科目列 -->
              <div
                class="cascade-column cascade-col-1"
                :class="{ 'active-column': activeColumn === 0 }"
              >
                <div class="column-title">科目</div>
                <q-item
                  v-for="(subject, index) in subjects"
                  :key="subject.id"
                  clickable
                  class="column-item"
                  :class="{
                    'item-selected': selectedSubject?.id === subject.id,
                    'item-active':
                      activeItemType === 'subject' && activeItemIndex === index
                  }"
                  @click="handleItemClick('subject', index)"
                  @mousedown="startLongPress('subject', index)"
                  @mouseup="cancelLongPress"
                  @mouseleave="cancelLongPress"
                  @touchstart="startLongPress('subject', index)"
                  @touchend="cancelLongPress"
                >
                  <div
                    v-if="
                      isEditing &&
                      activeItemType === 'subject' &&
                      activeItemIndex === index
                    "
                    class="edit-form"
                  >
                    <q-input
                      v-model="editingItemName"
                      outlined
                      dense
                      aria-label="输入内容"
                      @keyup.enter="saveEditSubject(subject, index)"
                    />
                    <input v-model="editingItemColor" type="color" />
                    <div class="edit-buttons">
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="primary"
                        class="btn-save"
                        @click="saveEditSubject(subject, index)"
                      >
                        ✓
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="btn-cancel"
                        @click="cancelEdit"
                      >
                        <Icon name="x" :size="16" />
                      </q-btn>
                    </div>
                  </div>
                  <template v-else>
                    <span
                      class="item-dot"
                      :style="{ background: subject.color || '#1976d2' }"
                    />
                    <span class="item-text">{{ subject.name }}</span>
                    <div
                      v-if="
                        activeItemType === 'subject' &&
                        activeItemIndex === index
                      "
                      class="item-actions"
                    >
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="action-btn edit-btn"
                        @click.stop="startEditSubject(subject, index)"
                      >
                        <Icon name="pencil" :size="16" />
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="negative"
                        class="action-btn delete-btn"
                        @click.stop="confirmDeleteSubject(subject)"
                      >
                        <Icon name="trash-2" :size="16" />
                      </q-btn>
                    </div>
                  </template>
                </q-item>
                <div v-if="subjects.length === 0" class="empty-column-item">
                  暂无科目
                </div>
              </div>

              <!-- 书籍列 -->
              <div
                v-show="selectedSubject"
                class="cascade-column cascade-col-2"
                :class="{
                  'active-column': activeColumn === 1,
                  'show-column': selectedSubject
                }"
              >
                <div class="column-title">书籍</div>
                <q-item
                  v-for="(book, index) in books"
                  :key="book"
                  clickable
                  class="column-item"
                  :class="{
                    'item-selected': selectedBook === book,
                    'item-active':
                      activeItemType === 'book' && activeItemIndex === index
                  }"
                  @click="handleItemClick('book', index)"
                  @mousedown="startLongPress('book', index)"
                  @mouseup="cancelLongPress"
                  @mouseleave="cancelLongPress"
                  @touchstart="startLongPress('book', index)"
                  @touchend="cancelLongPress"
                >
                  <div
                    v-if="
                      isEditing &&
                      activeItemType === 'book' &&
                      activeItemIndex === index
                    "
                    class="edit-form"
                  >
                    <q-input
                      v-model="editingItemName"
                      outlined
                      dense
                      aria-label="输入内容"
                      @keyup.enter="saveEditBook(book, index)"
                    />
                    <div class="edit-buttons">
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="primary"
                        class="btn-save"
                        @click="saveEditBook(book, index)"
                      >
                        ✓
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="btn-cancel"
                        @click="cancelEdit"
                      >
                        <Icon name="x" :size="16" />
                      </q-btn>
                    </div>
                  </div>
                  <template v-else>
                    <span class="item-text">{{ book }}</span>
                    <div
                      v-if="
                        activeItemType === 'book' && activeItemIndex === index
                      "
                      class="item-actions"
                    >
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="action-btn edit-btn"
                        @click.stop="startEditBook(book, index)"
                      >
                        <Icon name="pencil" :size="16" />
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="negative"
                        class="action-btn delete-btn"
                        @click.stop="confirmDeleteBook(book)"
                      >
                        <Icon name="trash-2" :size="16" />
                      </q-btn>
                    </div>
                  </template>
                </q-item>
                <div v-if="books.length === 0" class="empty-column-item">
                  暂无书籍
                </div>
              </div>

              <!-- 章节列 -->
              <div
                v-show="selectedBook"
                class="cascade-column cascade-col-3"
                :class="{
                  'active-column': activeColumn === 2,
                  'show-column': selectedBook
                }"
              >
                <div class="column-title">章节</div>
                <q-item
                  v-for="(chapter, index) in chapters"
                  :key="chapter"
                  clickable
                  class="column-item"
                  :class="{
                    'item-selected': selectedChapter === chapter,
                    'item-active':
                      activeItemType === 'chapter' && activeItemIndex === index
                  }"
                  @click="handleItemClick('chapter', index)"
                  @mousedown="startLongPress('chapter', index)"
                  @mouseup="cancelLongPress"
                  @mouseleave="cancelLongPress"
                  @touchstart="startLongPress('chapter', index)"
                  @touchend="cancelLongPress"
                >
                  <div
                    v-if="
                      isEditing &&
                      activeItemType === 'chapter' &&
                      activeItemIndex === index
                    "
                    class="edit-form"
                  >
                    <q-input
                      v-model="editingItemName"
                      outlined
                      dense
                      aria-label="输入内容"
                      @keyup.enter="saveEditChapter(chapter, index)"
                    />
                    <div class="edit-buttons">
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="primary"
                        class="btn-save"
                        @click="saveEditChapter(chapter, index)"
                      >
                        ✓
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="btn-cancel"
                        @click="cancelEdit"
                      >
                        <Icon name="x" :size="16" />
                      </q-btn>
                    </div>
                  </div>
                  <template v-else>
                    <span class="item-text">{{ chapter }}</span>
                    <div
                      v-if="
                        activeItemType === 'chapter' &&
                        activeItemIndex === index
                      "
                      class="item-actions"
                    >
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="action-btn edit-btn"
                        @click.stop="startEditChapter(chapter, index)"
                      >
                        <Icon name="pencil" :size="16" />
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="negative"
                        class="action-btn delete-btn"
                        @click.stop="confirmDeleteChapter(chapter)"
                      >
                        <Icon name="trash-2" :size="16" />
                      </q-btn>
                    </div>
                  </template>
                </q-item>
                <div v-if="chapters.length === 0" class="empty-column-item">
                  暂无章节
                </div>
              </div>

              <!-- 知识点列 -->
              <div
                v-show="selectedChapter"
                class="cascade-column cascade-col-4"
                :class="{
                  'active-column': activeColumn === 3,
                  'show-column': selectedChapter
                }"
              >
                <div class="column-title">知识点</div>
                <q-item
                  v-for="(knowledge, index) in knowledges"
                  :key="knowledge"
                  clickable
                  class="column-item"
                  :class="{
                    'item-selected': selectedKnowledge === knowledge,
                    'item-active':
                      activeItemType === 'knowledge' &&
                      activeItemIndex === index
                  }"
                  @click="handleItemClick('knowledge', index)"
                  @mousedown="startLongPress('knowledge', index)"
                  @mouseup="cancelLongPress"
                  @mouseleave="cancelLongPress"
                  @touchstart="startLongPress('knowledge', index)"
                  @touchend="cancelLongPress"
                >
                  <div
                    v-if="
                      isEditing &&
                      activeItemType === 'knowledge' &&
                      activeItemIndex === index
                    "
                    class="edit-form"
                  >
                    <q-input
                      v-model="editingItemName"
                      outlined
                      dense
                      aria-label="输入内容"
                      @keyup.enter="saveEditKnowledge(knowledge, index)"
                    />
                    <div class="edit-buttons">
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="primary"
                        class="btn-save"
                        @click="saveEditKnowledge(knowledge, index)"
                      >
                        ✓
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="btn-cancel"
                        @click="cancelEdit"
                      >
                        <Icon name="x" :size="16" />
                      </q-btn>
                    </div>
                  </div>
                  <template v-else>
                    <span class="item-text">{{ knowledge }}</span>
                    <div
                      v-if="
                        activeItemType === 'knowledge' &&
                        activeItemIndex === index
                      "
                      class="item-actions"
                    >
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        flat
                        class="action-btn edit-btn"
                        @click.stop="startEditKnowledge(knowledge, index)"
                      >
                        <Icon name="pencil" :size="16" />
                      </q-btn>
                      <q-btn
                        no-caps
                        unelevated
                        type="button"
                        color="negative"
                        class="action-btn delete-btn"
                        @click.stop="confirmDeleteKnowledge(knowledge)"
                      >
                        <Icon name="trash-2" :size="16" />
                      </q-btn>
                    </div>
                  </template>
                </q-item>
                <div v-if="knowledges.length === 0" class="empty-column-item">
                  暂无知识点
                </div>
              </div>
            </div>

            <!-- 删除确认弹窗 -->
            <q-dialog :model-value="showDeleteConfirm" persistent>
              <div class="delete-confirm-modal">
                <h3>确认删除</h3>
                <p>确定要删除 {{ deleteItemInfo?.name }} 吗？</p>
                <p
                  v-if="deleteItemInfo?.type !== 'knowledge'"
                  class="warning-text"
                >
                  这将同时删除所有下级内容！
                </p>
                <div class="modal-buttons">
                  <q-btn
                    no-caps
                    unelevated
                    type="button"
                    color="negative"
                    class="btn-danger"
                    @click="executeDelete"
                  >
                    删除
                  </q-btn>
                  <q-btn
                    no-caps
                    unelevated
                    type="button"
                    flat
                    class="btn-secondary"
                    @click="cancelDelete"
                  >
                    取消
                  </q-btn>
                </div>
              </div>
            </q-dialog>
          </div>
        </div>
      </q-card>

      <!-- 错因分布 -->
      <q-card flat bordered class="chart-section">
        <div class="section-header">
          <h3>错因分布</h3>
          <q-btn
            no-caps
            unelevated
            type="button"
            flat
            class="manage-btn"
            @click="openManageModal"
          >
            <Icon name="settings" :size="18" class="manage-icon" />
            <span>管理</span>
          </q-btn>
        </div>
        <div
          v-if="errorTagDistribution.length > 0"
          class="error-tag-distribution"
        >
          <div class="donut-chart-wrapper">
            <svg class="donut-chart" viewBox="0 0 100 100">
              <!-- 背景圆环 -->
              <circle
                cx="50"
                cy="50"
                r="40"
                fill="none"
                stroke="var(--border-color)"
                stroke-width="20"
              />
              <!-- 扇区 -->
              <circle
                v-for="(segment, index) in donutSegments"
                :key="index"
                class="donut-segment"
                :class="{ 'segment-hovered': hoveredIndex === index }"
                cx="50"
                cy="50"
                :r="hoveredIndex === index ? 42 : 40"
                fill="none"
                :stroke="segment.color"
                :stroke-width="hoveredIndex === index ? 24 : 20"
                :stroke-dasharray="`${segment.percent} ${251.33}`"
                :stroke-dashoffset="`${segment.offset}`"
                :stroke-linecap="
                  (segment.linecap as
                    'butt' | 'round' | 'square' | 'inherit') || 'butt'
                "
                @mouseenter="hoveredIndex = index"
                @mouseleave="hoveredIndex = -1"
              />
            </svg>
            <!-- 中心显示总数量 -->
            <div class="donut-center">
              <div class="donut-center-count">
                {{ totalTags }}
              </div>
              <div class="donut-center-label">题目错因总数</div>
            </div>
          </div>
          <div class="error-tag-legend">
            <div
              v-for="(tag, index) in errorTagDistribution"
              :key="index"
              class="error-tag-item"
              :class="{ 'item-highlighted': hoveredIndex === index }"
              @mouseenter="hoveredIndex = index"
              @mouseleave="hoveredIndex = -1"
            >
              <span class="tag-color-dot" :style="{ background: tag.color }" />
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-count">{{ tag.count }}</span>
              <span class="tag-percentage"
                >{{ calculatePercentage(tag.count) }}%</span
              >
            </div>
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无错因数据</div>
        </div>
      </q-card>

      <!-- 错因管理弹窗 -->
      <q-dialog
        class="notebook-dialog"
        :model-value="showManageModal"
        @hide="closeManageModal"
      >
        <div class="modal-container">
          <div class="modal-header">
            <h3 class="modal-title">错因管理</h3>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="modal-close"
              @click="closeManageModal"
            >
              ×
            </q-btn>
          </div>
          <div class="modal-body">
            <div v-if="manageErrorTags.length === 0" class="empty-list">
              <div class="empty-text">暂无错因标签</div>
            </div>
            <div v-else class="tag-list">
              <div
                v-for="(tag, index) in manageErrorTags"
                :key="tag.name"
                class="tag-item"
              >
                <div class="tag-info">
                  <span
                    class="tag-dot"
                    :style="{
                      background:
                        editingTagIndex === index ? editingTagColor : tag.color
                    }"
                  />
                  <span
                    v-if="editingTagIndex !== index"
                    class="tag-name-text"
                    >{{ tag.name }}</span
                  >
                  <template v-else>
                    <q-input
                      v-model="editingTagName"
                      outlined
                      dense
                      aria-label="输入内容"
                      type="text"
                      class="tag-input"
                      @keyup.enter="saveTagEdit(index)"
                      @keyup.escape="cancelTagEdit"
                    />
                    <input
                      v-model="editingTagColor"
                      type="color"
                      class="tag-color-input"
                    />
                  </template>
                </div>
                <div class="tag-actions">
                  <div v-if="editingTagIndex !== index" class="actions-group">
                    <q-btn
                      no-caps
                      unelevated
                      type="button"
                      flat
                      class="action-btn edit-btn"
                      @click="startEditTag(index)"
                    >
                      <Icon name="pencil" :size="16" />
                    </q-btn>
                    <q-btn
                      no-caps
                      unelevated
                      type="button"
                      color="negative"
                      class="action-btn delete-btn"
                      @click="confirmDeleteTag(tag)"
                    >
                      <Icon name="trash-2" :size="16" />
                    </q-btn>
                  </div>
                  <div v-else class="actions-group">
                    <q-btn
                      no-caps
                      unelevated
                      type="button"
                      color="primary"
                      class="action-btn save-btn"
                      @click="saveTagEdit(index)"
                    >
                      <span>✓</span>
                    </q-btn>
                    <q-btn
                      no-caps
                      unelevated
                      type="button"
                      flat
                      class="action-btn cancel-btn"
                      @click="cancelTagEdit"
                    >
                      <Icon name="x" :size="16" />
                    </q-btn>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="btn btn-secondary"
              @click="closeManageModal"
            >
              关闭
            </q-btn>
          </div>
        </div>
      </q-dialog>

      <!-- 错因标签删除确认弹窗 -->
      <q-dialog
        class="notebook-dialog"
        :model-value="showTagDeleteConfirm"
        @hide="showTagDeleteConfirm = false"
      >
        <div class="modal-container confirm-modal">
          <div class="modal-header">
            <h3 class="modal-title">确认删除</h3>
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="modal-close"
              @click="showTagDeleteConfirm = false"
            >
              ×
            </q-btn>
          </div>
          <div class="modal-body">
            <p class="confirm-text">
              确定要删除错因标签「{{ deleteTagName }}」吗？
            </p>
            <p class="confirm-hint">此操作将删除所有题目的该标签</p>
          </div>
          <div class="modal-footer">
            <q-btn
              no-caps
              unelevated
              type="button"
              flat
              class="btn btn-secondary"
              @click="showTagDeleteConfirm = false"
            >
              取消
            </q-btn>
            <q-btn
              no-caps
              unelevated
              type="button"
              color="negative"
              class="btn btn-danger"
              @click="executeDeleteTag"
            >
              删除
            </q-btn>
          </div>
        </div>
      </q-dialog>

      <!-- 科目 × 难度热力图 -->
      <q-card flat bordered class="chart-section">
        <h3>科目 × 难度热力图</h3>
        <div class="heatmap-hint">
          颜色越深表示该区间题目越多，难度区间动态10等分
        </div>
        <div
          v-if="heatmapData.length > 0"
          class="heatmap-wrapper"
          @click.self="selectedCell = null"
        >
          <table class="heatmap-table">
            <thead>
              <tr>
                <th class="row-header">科目</th>
                <th v-for="b in 10" :key="b" class="col-header">
                  {{ b }}
                </th>
                <th class="row-header total-col">合计</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, ri) in heatmapData" :key="ri">
                <td class="row-label" :title="row.subject">
                  {{ row.subjectShort }}
                </td>
                <td
                  v-for="(cell, ci) in row.buckets"
                  :key="ci"
                  class="heat-cell"
                  :class="{
                    selected: selectedCell?.ri === ri && selectedCell?.ci === ci
                  }"
                  :style="getHeatStyle(cell.count, heatmapMax)"
                  @click="toggleCell(ri, ci)"
                >
                  {{ cell.count || '' }}
                </td>
                <td class="row-total">
                  {{ row.total }}
                </td>
              </tr>
            </tbody>
          </table>

          <!-- 点击详情弹窗 -->
          <div v-if="selectedCell" class="cell-detail">
            <div class="cell-detail-row">科目: {{ selectedCell.subject }}</div>
            <div class="cell-detail-row">
              难度区间: {{ selectedCell.label }}
            </div>
            <div class="cell-detail-row">
              题目数量: <strong>{{ selectedCell.count }}</strong>
            </div>
          </div>
          <div class="heatmap-legend">
            <span>少</span>
            <span
              v-for="l in 5"
              :key="l"
              class="legend-block"
              :style="{ background: `rgba(33, 150, 243, ${0.1 + l * 0.18})` }"
            />
            <span>多</span>
          </div>
          <div v-if="difficultyRange" class="difficulty-range">
            难度范围: {{ difficultyRange.min.toFixed(4) }} ~
            {{ difficultyRange.max.toFixed(4) }}
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无足够数据生成热力图</div>
        </div>
      </q-card>

      <!-- 复习间隔分布 -->
      <q-card flat bordered class="chart-section">
        <h3>复习间隔分布</h3>
        <div v-if="intervalBuckets.length > 0" class="interval-chart">
          <div class="interval-bars">
            <div
              v-for="(bucket, index) in intervalBuckets"
              :key="index"
              class="interval-bar-group"
            >
              <div class="bar-value">
                {{ bucket.count }}
              </div>
              <div
                class="bar-fill"
                :style="{
                  height: getIntervalBarHeight(bucket.count) + '%',
                  background: bucket.color
                }"
              />
              <div class="bar-label">
                {{ bucket.label }}
              </div>
            </div>
          </div>
        </div>
        <div v-else class="empty-state">
          <div class="empty-description">暂无间隔数据</div>
        </div>
      </q-card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import {
  deleteSubject as removeSubject,
  deleteTag,
  getLibraryStatistics,
  updateSource,
  updateSubject as editSubject,
  updateTag
} from '../api'
import { deleteSources } from '../api'
import { useCountUp } from '../composables/useCountUp'
import { useLatestRequest } from '../composables/useLatestRequest'
import { loadQuestionLibrary } from '../services/questionQueries'
import { createSourceCatalog } from '../services/sourceCatalog'
import type { Question, SrsData } from '../types'
import type { Source, Subject, Tag } from '../types'
import { showAlert } from '../utils/dialog'
import { timestampSeconds } from '../utils/questionDisplay'

// ==================== 状态 ====================
const loading = ref(true)
const loadError = ref('')

// ==================== 科目分布级联选择状态 ====================
const selectedSubject = ref<Subject | null>(null)
const selectedBook = ref<string | null>(null)
const selectedChapter = ref<string | null>(null)
const selectedKnowledge = ref<string | null>(null)
const books = ref<string[]>([])
const chapters = ref<string[]>([])
const knowledges = ref<string[]>([])
const activeColumn = ref<number>(0)
const showCascade = ref<boolean>(false)
const cascadeContainer = ref<HTMLElement | null>(null)

// 原始数据
const questionTotal = ref(0)
const dueCount = ref(0)
const srsStats = ref({
  total: 0,
  dueCount: 0,
  newCardCount: 0,
  averageStability: 0,
  averageDifficulty: 0,
  totalReviews: 0
})
const subjects = ref<Subject[]>([])
const questions = ref<Question[]>([])
const allCards = ref<SrsData[]>([])
const errorTags = ref<Tag[]>([])
const hoveredIndex = ref(-1)

// 错因管理相关状态
const showManageModal = ref(false)
const showTagDeleteConfirm = ref(false)

// 唯一错因标签管理数据结构
interface UniqueErrorTag {
  name: string
  color: string
  ids: string[] // 所有具有相同name的标签ID
}

const manageErrorTags = ref<UniqueErrorTag[]>([])
const editingTagIndex = ref<number | null>(null)
const editingTagName = ref('')
const editingTagColor = ref('')
const deleteTagName = ref('')

// 科目/来源管理相关状态
const allSources = ref<Source[]>([])
const { getBooks, getChapters, getKnowledges } = createSourceCatalog(
  () => allSources.value
)
const activeItemType = ref<'subject' | 'book' | 'chapter' | 'knowledge' | null>(
  null
)
const activeItemIndex = ref<number | null>(null)
const longPressTimer = ref<number | null>(null)
const isEditing = ref(false)
const editingItemName = ref('')
const editingItemColor = ref('')
const justFinishedLongPress = ref(false)
const isInLongPressMode = ref(false)
const showDeleteConfirm = ref(false)
const deleteItemInfo = ref<{
  type: 'subject' | 'book' | 'chapter' | 'knowledge'
  id?: string
  name: string
  subjectId?: string
  book?: string
  chapter?: string
} | null>(null)

// 难度区间标注

// ==================== 科目分布 ====================
interface SubjectDistItem {
  name: string
  count: number
  percent: number
  color: string
}

// Question only owns sourceId. Its subject is always projected from the
// current Source collection so hierarchy mutations cannot leave stale stats.
const sourceSubjectById = computed(
  () =>
    new Map(
      allSources.value.map((source) => [source.id, source.subjectId ?? ''])
    )
)

const questionSubjectId = (question: Pick<Question, 'sourceId'>) =>
  question.sourceId
    ? (sourceSubjectById.value.get(question.sourceId) ?? '')
    : ''

const subjectDistribution = computed<SubjectDistItem[]>(() => {
  const subjectMap = new Map<string, { name: string; color: string }>()
  subjects.value.forEach((s) =>
    subjectMap.set(s.id, { name: s.name, color: s.color || '#1976d2' })
  )

  const countMap = new Map<string, number>()
  questions.value.forEach((q) => {
    const sid = questionSubjectId(q)
    countMap.set(sid, (countMap.get(sid) || 0) + 1)
  })

  const total = questionTotal.value
  if (total === 0) return []

  return Array.from(countMap.entries())
    .map(([id, count]) => {
      const subjectInfo = subjectMap.get(id) || {
        name: '未分类',
        color: '#1976d2'
      }
      return {
        name: subjectInfo.name,
        count,
        percent: Math.round((count / total) * 100),
        color: subjectInfo.color
      }
    })
    .sort((a, b) => b.count - a.count)
})

// ==================== 概览卡片 ====================
const overview = computed(() => ({
  total: questionTotal.value,
  dueCount: dueCount.value,
  currentMemory: Math.max(0, questionTotal.value - dueCount.value),
  newCards: srsStats.value.newCardCount
}))

// 数字滚动计数
const totalRef = computed(() => overview.value.total)
const dueRef = computed(() => overview.value.dueCount)
const memoryRef = computed(() => overview.value.currentMemory)
const newCardsRef = computed(() => overview.value.newCards)

const { displayValue: animatedTotal } = useCountUp(totalRef)
const { displayValue: animatedDue } = useCountUp(dueRef)
const { displayValue: animatedMemory } = useCountUp(memoryRef)
const { displayValue: animatedNewCards } = useCountUp(newCardsRef)

// ==================== 科目 × 难度热力图 ====================

interface HeatmapBucket {
  count: number
  label: string
  rangeMin: number
  rangeMax: number
}

interface HeatmapRow {
  subject: string
  subjectShort: string
  buckets: HeatmapBucket[]
  total: number
}

const heatmap = computed<{
  rows: HeatmapRow[]
  range: { min: number; max: number } | null
}>(() => {
  if (allCards.value.length === 0) return { rows: [], range: null }

  // 建立 questionId → subjectId 映射
  const qToSubject = new Map<string, string>()
  questions.value.forEach((q) => qToSubject.set(q.id, questionSubjectId(q)))

  // 建立 subjectId → name 映射
  const subjectNameMap = new Map<string, string>()
  subjects.value.forEach((s) => subjectNameMap.set(s.id, s.name))

  // 收集有 subject 关联的卡片，收集难度值
  const validCards: { subjectId: string; difficulty: number }[] = []
  const difficulties: number[] = []

  for (const card of allCards.value) {
    const subjectId = qToSubject.get(card.questionId)
    if (subjectId === undefined) continue
    validCards.push({ subjectId, difficulty: card.difficulty })
    difficulties.push(card.difficulty)
  }

  if (validCards.length === 0 || difficulties.length === 0)
    return { rows: [], range: null }

  const minDiff = Math.min(...difficulties)
  const maxDiff = Math.max(...difficulties)

  const range = maxDiff - minDiff
  const bucketSize = range / 10

  // 按 subject 分组
  const subjectGroups = new Map<string, number[]>()
  for (const vc of validCards) {
    const arr = subjectGroups.get(vc.subjectId) || []
    arr.push(vc.difficulty)
    subjectGroups.set(vc.subjectId, arr)
  }

  // 构建矩阵
  const rows: HeatmapRow[] = []

  for (const [subjId, diffs] of subjectGroups) {
    const buckets: HeatmapBucket[] = []
    for (let i = 0; i < 10; i++) {
      const lo = minDiff + i * bucketSize
      const hi = i === 9 ? maxDiff + 0.001 : minDiff + (i + 1) * bucketSize
      const count = diffs.filter((d) => d >= lo && d < hi).length
      buckets.push({
        count,
        label: `${lo.toFixed(4)}-${i === 9 ? maxDiff.toFixed(4) : (minDiff + (i + 1) * bucketSize).toFixed(4)}`,
        rangeMin: lo,
        rangeMax: hi
      })
    }
    const name = subjectNameMap.get(subjId) || '未分类'
    rows.push({
      subject: name,
      subjectShort: name,
      buckets,
      total: diffs.length
    })
  }

  rows.sort((a, b) => b.total - a.total)
  return { rows, range: { min: minDiff, max: maxDiff } }
})
const heatmapData = computed(() => heatmap.value.rows)
const difficultyRange = computed(() => heatmap.value.range)

// ==================== 错因分布 ====================
interface ErrorTagDistItem {
  name: string
  count: number
  color: string
}

const errorTagDistribution = computed<ErrorTagDistItem[]>(() => {
  const tagCountMap = new Map<string, { count: number; color: string }>()

  errorTags.value.forEach((tag) => {
    // 过滤掉软删除的标签
    if (tag.name.startsWith('[已删除]')) {
      return
    }

    const existing = tagCountMap.get(tag.name)
    if (existing) {
      existing.count++
    } else {
      tagCountMap.set(tag.name, { count: 1, color: tag.color })
    }
  })

  return Array.from(tagCountMap.entries())
    .map(([name, data]) => ({ name, count: data.count, color: data.color }))
    .sort((a, b) => b.count - a.count)
})

const heatmapMax = computed(() => {
  let max = 0
  for (const row of heatmapData.value) {
    for (const b of row.buckets) {
      if (b.count > max) max = b.count
    }
  }
  return max
})

// ==================== 热力图点击详情 ====================
const selectedCell = ref<{
  ri: number
  ci: number
  subject: string
  label: string
  count: number
} | null>(null)

function toggleCell(ri: number, ci: number) {
  const row = heatmapData.value[ri]
  if (!row) return
  const cell = row.buckets[ci]
  if (!cell) return

  if (selectedCell.value?.ri === ri && selectedCell.value?.ci === ci) {
    selectedCell.value = null
  } else {
    selectedCell.value = {
      ri,
      ci,
      subject: row.subject,
      label: cell.label,
      count: cell.count
    }
  }
}

const getHeatStyle = (count: number, max: number) => {
  if (count === 0 || max === 0) return { background: 'transparent' }
  const intensity = Math.min(1, count / max)
  return { background: `rgba(33, 150, 243, ${0.08 + intensity * 0.82})` }
}

// ==================== 复习间隔分布 ====================
interface IntervalBucket {
  label: string
  count: number
  color: string
}

const intervalBuckets = computed<IntervalBucket[]>(() => {
  const now = Math.floor(Date.now() / 1000)

  const buckets: IntervalBucket[] = [
    { label: '新卡片', count: 0, color: '#bdbdbd' },
    { label: '已到期', count: 0, color: '#ef5350' },
    { label: '1-3天', count: 0, color: '#ff9800' },
    { label: '4-7天', count: 0, color: '#ffc107' },
    { label: '8-14天', count: 0, color: '#66bb6a' },
    { label: '15-30天', count: 0, color: '#42a5f5' },
    { label: '30天+', count: 0, color: '#7e57c2' }
  ]

  for (const card of allCards.value) {
    if (card.reviewCount <= 1 || card.nextReviewAt === null) {
      buckets[0].count++ // 新卡片（FSRS-5 初始 reviewCount=1）
      continue
    }
    const daysUntil = (timestampSeconds(card.nextReviewAt) - now) / 86400
    if (daysUntil <= 0) buckets[1].count++
    else if (daysUntil <= 3) buckets[2].count++
    else if (daysUntil <= 7) buckets[3].count++
    else if (daysUntil <= 14) buckets[4].count++
    else if (daysUntil <= 30) buckets[5].count++
    else buckets[6].count++
  }

  return buckets
})

const intervalMaxCount = computed(() =>
  Math.max(...intervalBuckets.value.map((b) => b.count), 1)
)

const getIntervalBarHeight = (count: number) =>
  (count / intervalMaxCount.value) * 100

// ==================== 数据加载 ====================
const beginLoad = useLatestRequest()
async function loadData() {
  const isCurrent = beginLoad()
  loading.value = true
  loadError.value = ''
  try {
    const [library, statistics] = await Promise.all([
      loadQuestionLibrary(),
      getLibraryStatistics(new Date().toISOString())
    ])
    if (!isCurrent()) return
    questionTotal.value = statistics.questionTotal
    dueCount.value = statistics.dueCount
    srsStats.value = { ...statistics, total: statistics.cardTotal }
    subjects.value = library.subjects
    allSources.value = library.sources
    questions.value = library.items
    allCards.value = library.srs
    // Distribution counts relationships, while editing targets reusable Tag IDs.
    errorTags.value = library.items.flatMap((question) => question.tags)
  } catch (error) {
    if (isCurrent()) loadError.value = String(error)
  } finally {
    if (isCurrent()) loading.value = false
  }
}

loadData()

// ==================== 级联响应式布局 ====================
// 当展开的列触及容器右端时，自动缩小列间距以适配宽度
function updateCascadeGap() {
  if (!showCascade.value || !cascadeContainer.value) return

  const wrapper = cascadeContainer.value.querySelector(
    '.cascade-column-wrapper'
  ) as HTMLElement | null
  if (!wrapper) return

  const containerWidth = cascadeContainer.value.clientWidth - 2
  const colWidth = 160

  const visibleCount =
    1 +
    (selectedSubject.value ? 1 : 0) +
    (selectedBook.value ? 1 : 0) +
    (selectedChapter.value ? 1 : 0)

  if (visibleCount <= 1) {
    wrapper.style.removeProperty('--col-gap')
    return
  }

  const minGap = 40
  const maxGap = 80
  const idealGap = Math.round(
    Math.max(
      minGap,
      Math.min(maxGap, (containerWidth - colWidth) / (visibleCount - 1))
    )
  )

  wrapper.style.setProperty('--col-gap', `${idealGap}px`)
}

watch(
  [showCascade, selectedSubject, selectedBook, selectedChapter],
  () => {
    if (showCascade.value) {
      updateCascadeGap()
    }
  },
  { flush: 'post' }
)

function onWindowResize() {
  if (showCascade.value) {
    updateCascadeGap()
  }
}

// ==================== 科目分布级联选择逻辑 ====================
const handleSelectSubject = async (subject: Subject) => {
  selectedSubject.value = subject
  selectedBook.value = null
  selectedChapter.value = null
  selectedKnowledge.value = null
  chapters.value = []
  knowledges.value = []
  activeColumn.value = 1
  showCascade.value = true

  try {
    books.value = getBooks(subject.id)
  } catch (error) {
    console.error('获取书籍失败:', error)
    books.value = []
  }
}

const handleSelectSubjectByName = async (subjectName: string) => {
  const subject = subjects.value.find((s) => s.name === subjectName)
  if (subject) {
    await handleSelectSubject(subject)
  }
}

const openManageCascade = async () => {
  showCascade.value = true
  // 如果有科目但没有选中的科目，自动选中第一个
  if (subjects.value.length > 0 && !selectedSubject.value) {
    await handleSelectSubject(subjects.value[0])
  }
}

const handleSelectBook = async (book: string) => {
  selectedBook.value = book
  selectedChapter.value = null
  selectedKnowledge.value = null
  knowledges.value = []
  activeColumn.value = 2

  try {
    chapters.value = getChapters(book, selectedSubject.value?.id)
  } catch (error) {
    console.error('获取章节失败:', error)
    chapters.value = []
  }
}

const handleSelectChapter = async (chapter: string) => {
  selectedChapter.value = chapter
  selectedKnowledge.value = null
  activeColumn.value = 3

  try {
    knowledges.value = getKnowledges(
      selectedBook.value!,
      chapter,
      selectedSubject.value?.id
    )
  } catch (error) {
    console.error('获取知识点失败:', error)
    knowledges.value = []
  }
}

const handleSelectKnowledge = (knowledge: string) => {
  selectedKnowledge.value = knowledge
}

const closeCascade = () => {
  showCascade.value = false
  selectedSubject.value = null
  selectedBook.value = null
  selectedChapter.value = null
  selectedKnowledge.value = null
  books.value = []
  chapters.value = []
  knowledges.value = []
  activeColumn.value = 0
  activeItemType.value = null
  activeItemIndex.value = null
  isEditing.value = false
}

// ==================== 科目/来源长按和管理功能 ====================
let longPressResetTimer: ReturnType<typeof setTimeout> | undefined

function startLongPress(
  type: 'subject' | 'book' | 'chapter' | 'knowledge',
  index: number
) {
  if (longPressTimer.value) {
    clearTimeout(longPressTimer.value)
  }
  longPressTimer.value = window.setTimeout(() => {
    activeItemType.value = type
    activeItemIndex.value = index
    isInLongPressMode.value = true
    justFinishedLongPress.value = true

    // 根据标签类型关闭下级列
    if (type === 'subject') {
      // 关闭书籍、章节、知识点
      selectedBook.value = null
      selectedChapter.value = null
      selectedKnowledge.value = null
      chapters.value = []
      knowledges.value = []
      activeColumn.value = 0
    } else if (type === 'book') {
      // 关闭章节、知识点
      selectedChapter.value = null
      selectedKnowledge.value = null
      knowledges.value = []
      activeColumn.value = 1
    } else if (type === 'chapter') {
      // 关闭知识点
      selectedKnowledge.value = null
      activeColumn.value = 2
    }

    // 300ms 后重置标志，避免后续点击被影响
    clearTimeout(longPressResetTimer)
    longPressResetTimer = setTimeout(() => {
      justFinishedLongPress.value = false
    }, 300)
  }, 500)
}

function cancelLongPress() {
  if (longPressTimer.value) {
    clearTimeout(longPressTimer.value)
    longPressTimer.value = null
  }
}

function handleItemClick(
  type: 'subject' | 'book' | 'chapter' | 'knowledge',
  index: number
) {
  cancelLongPress()

  // 如果刚完成长按，则跳过这次点击处理
  if (justFinishedLongPress.value) {
    isInLongPressMode.value = false
    return
  }

  // 如果正在编辑，则不触发点击事件
  if (isEditing.value) {
    return
  }

  // 如果当前项目已经激活，则取消激活，不触发选择
  if (activeItemType.value === type && activeItemIndex.value === index) {
    activeItemType.value = null
    activeItemIndex.value = null
    isEditing.value = false
    isInLongPressMode.value = false
    return
  }

  // 清除之前的激活状态
  activeItemType.value = null
  activeItemIndex.value = null
  isEditing.value = false

  // 如果是长按模式刚结束，则不关闭下级，只清除长按模式标志
  if (isInLongPressMode.value) {
    isInLongPressMode.value = false
    return
  }

  // 点击科目标签
  if (type === 'subject') {
    const subject = subjects.value[index]
    if (selectedSubject.value?.id === subject.id) {
      // 点击了已选择的科目 - 关闭下一级（书籍）
      selectedBook.value = null
      selectedChapter.value = null
      selectedKnowledge.value = null
      chapters.value = []
      knowledges.value = []
      activeColumn.value = 1
      return
    } else {
      // 点击了未选择的科目 - 选中并展开
      handleSelectSubject(subject)
    }
  }
  // 点击书籍标签
  else if (type === 'book') {
    const book = books.value[index]
    if (selectedBook.value === book) {
      // 点击了已选择的书籍 - 关闭下一级（章节）
      selectedChapter.value = null
      selectedKnowledge.value = null
      knowledges.value = []
      activeColumn.value = 2
      return
    } else {
      // 点击了未选择的书籍 - 选中并展开
      handleSelectBook(book)
    }
  }
  // 点击章节标签
  else if (type === 'chapter') {
    const chapter = chapters.value[index]
    if (selectedChapter.value === chapter) {
      // 点击了已选择的章节 - 关闭下一级（知识点）
      selectedKnowledge.value = null
      activeColumn.value = 3
      return
    } else {
      // 点击了未选择的章节 - 选中并展开
      handleSelectChapter(chapter)
    }
  }
  // 点击知识点标签
  else if (type === 'knowledge') {
    const knowledge = knowledges.value[index]
    handleSelectKnowledge(knowledge)
  }
}

// 编辑功能
async function startEditSubject(subject: Subject, index: number) {
  activeItemType.value = 'subject'
  activeItemIndex.value = index
  isEditing.value = true
  editingItemName.value = subject.name
  editingItemColor.value = subject.color || '#1976d2'
}

async function saveEditSubject(subject: Subject, index: number) {
  try {
    const updatedSubject = await editSubject({
      ...subject,
      name: editingItemName.value,
      color: editingItemColor.value
    })
    subjects.value[index] = updatedSubject
    isEditing.value = false
    activeItemType.value = null
    activeItemIndex.value = null
    isInLongPressMode.value = false
  } catch (error) {
    console.error('更新科目失败:', error)
    showAlert('更新失败')
  }
}

async function startEditBook(bookName: string, index: number) {
  activeItemType.value = 'book'
  activeItemIndex.value = index
  isEditing.value = true
  editingItemName.value = bookName
}

async function saveEditBook(oldName: string, index: number) {
  try {
    const subjectId = selectedSubject.value?.id
    if (!subjectId) return

    const bookSources = allSources.value.filter(
      (s) => s.subjectId === subjectId && s.book === oldName
    )

    for (const source of bookSources) {
      await updateSource({
        ...source,
        book: editingItemName.value
      })
    }

    // 更新本地数据
    allSources.value = allSources.value.map((s) => {
      if (s.subjectId === subjectId && s.book === oldName) {
        return { ...s, book: editingItemName.value }
      }
      return s
    })

    books.value[index] = editingItemName.value
    if (selectedBook.value === oldName) {
      selectedBook.value = editingItemName.value
    }

    isEditing.value = false
    activeItemType.value = null
    activeItemIndex.value = null
    isInLongPressMode.value = false
  } catch (error) {
    console.error('更新书籍失败:', error)
    showAlert('更新失败')
  }
}

async function startEditChapter(chapterName: string, index: number) {
  activeItemType.value = 'chapter'
  activeItemIndex.value = index
  isEditing.value = true
  editingItemName.value = chapterName
}

async function saveEditChapter(oldName: string, index: number) {
  try {
    const subjectId = selectedSubject.value?.id
    const bookName = selectedBook.value
    if (!subjectId || !bookName) return

    const chapterSources = allSources.value.filter(
      (s) =>
        s.subjectId === subjectId &&
        s.book === bookName &&
        s.chapter === oldName
    )

    for (const source of chapterSources) {
      await updateSource({
        ...source,
        chapter: editingItemName.value
      })
    }

    // 更新本地数据
    allSources.value = allSources.value.map((s) => {
      if (
        s.subjectId === subjectId &&
        s.book === bookName &&
        s.chapter === oldName
      ) {
        return { ...s, chapter: editingItemName.value }
      }
      return s
    })

    chapters.value[index] = editingItemName.value
    if (selectedChapter.value === oldName) {
      selectedChapter.value = editingItemName.value
    }

    isEditing.value = false
    activeItemType.value = null
    activeItemIndex.value = null
    isInLongPressMode.value = false
  } catch (error) {
    console.error('更新章节失败:', error)
    showAlert('更新失败')
  }
}

async function startEditKnowledge(knowledgeName: string, index: number) {
  activeItemType.value = 'knowledge'
  activeItemIndex.value = index
  isEditing.value = true
  editingItemName.value = knowledgeName
}

async function saveEditKnowledge(oldName: string, index: number) {
  try {
    const subjectId = selectedSubject.value?.id
    const bookName = selectedBook.value
    const chapterName = selectedChapter.value
    if (!subjectId || !bookName || !chapterName) return

    const knowledgeSources = allSources.value.filter(
      (s) =>
        s.subjectId === subjectId &&
        s.book === bookName &&
        s.chapter === chapterName &&
        s.knowledge === oldName
    )

    for (const source of knowledgeSources) {
      await updateSource({
        ...source,
        knowledge: editingItemName.value
      })
    }

    // 更新本地数据
    allSources.value = allSources.value.map((s) => {
      if (
        s.subjectId === subjectId &&
        s.book === bookName &&
        s.chapter === chapterName &&
        s.knowledge === oldName
      ) {
        return { ...s, knowledge: editingItemName.value }
      }
      return s
    })

    knowledges.value[index] = editingItemName.value
    if (selectedKnowledge.value === oldName) {
      selectedKnowledge.value = editingItemName.value
    }

    isEditing.value = false
    activeItemType.value = null
    activeItemIndex.value = null
    isInLongPressMode.value = false
  } catch (error) {
    console.error('更新知识点失败:', error)
    showAlert('更新失败')
  }
}

// 删除功能 - 递归删除所有子项
async function confirmDeleteSubject(subject: Subject) {
  deleteItemInfo.value = {
    type: 'subject',
    id: subject.id,
    name: subject.name
  }
  showDeleteConfirm.value = true
}

async function confirmDeleteBook(bookName: string) {
  deleteItemInfo.value = {
    type: 'book',
    name: bookName,
    subjectId: selectedSubject.value?.id
  }
  showDeleteConfirm.value = true
}

async function confirmDeleteChapter(chapterName: string) {
  deleteItemInfo.value = {
    type: 'chapter',
    name: chapterName,
    subjectId: selectedSubject.value?.id,
    book: selectedBook.value ?? undefined
  }
  showDeleteConfirm.value = true
}

async function confirmDeleteKnowledge(knowledgeName: string) {
  deleteItemInfo.value = {
    type: 'knowledge',
    name: knowledgeName,
    subjectId: selectedSubject.value?.id,
    book: selectedBook.value ?? undefined,
    chapter: selectedChapter.value ?? undefined
  }
  showDeleteConfirm.value = true
}

async function executeDelete() {
  try {
    if (!deleteItemInfo.value) return

    const { type, id, name, subjectId, book, chapter } = deleteItemInfo.value

    if (type === 'subject' && id) {
      // 应用层会在同一事务内删除科目，并清空相关来源的科目关联。
      await removeSubject(id)

      // 更新本地数据
      allSources.value = allSources.value.map((source) =>
        source.subjectId === id ? { ...source, subjectId: null } : source
      )
      subjects.value = subjects.value.filter((s) => s.id !== id)

      if (selectedSubject.value?.id === id) {
        closeCascade()
      }
    } else if (type === 'book' && subjectId) {
      // 删除书籍 - 递归删除所有相关章节和知识点
      const bookSources = allSources.value.filter(
        (s) => s.subjectId === subjectId && s.book === name
      )
      await deleteSources(bookSources.map((source) => source.id))

      // 更新本地数据
      allSources.value = allSources.value.filter(
        (s) => !(s.subjectId === subjectId && s.book === name)
      )
      books.value = books.value.filter((b) => b !== name)

      if (selectedBook.value === name) {
        selectedBook.value = null
        selectedChapter.value = null
        selectedKnowledge.value = null
        chapters.value = []
        knowledges.value = []
        activeColumn.value = 1
      }
    } else if (type === 'chapter' && subjectId && book) {
      // 删除章节 - 递归删除所有相关知识点
      const chapterSources = allSources.value.filter(
        (s) =>
          s.subjectId === subjectId && s.book === book && s.chapter === name
      )
      await deleteSources(chapterSources.map((source) => source.id))

      // 更新本地数据
      allSources.value = allSources.value.filter(
        (s) =>
          !(s.subjectId === subjectId && s.book === book && s.chapter === name)
      )
      chapters.value = chapters.value.filter((c) => c !== name)

      if (selectedChapter.value === name) {
        selectedChapter.value = null
        selectedKnowledge.value = null
        knowledges.value = []
        activeColumn.value = 2
      }
    } else if (type === 'knowledge' && subjectId && book && chapter) {
      // 删除知识点
      const knowledgeSources = allSources.value.filter(
        (s) =>
          s.subjectId === subjectId &&
          s.book === book &&
          s.chapter === chapter &&
          s.knowledge === name
      )
      await deleteSources(knowledgeSources.map((source) => source.id))

      // 更新本地数据
      allSources.value = allSources.value.filter(
        (s) =>
          !(
            s.subjectId === subjectId &&
            s.book === book &&
            s.chapter === chapter &&
            s.knowledge === name
          )
      )
      knowledges.value = knowledges.value.filter((k) => k !== name)

      if (selectedKnowledge.value === name) {
        selectedKnowledge.value = null
      }
    }

    showDeleteConfirm.value = false
    deleteItemInfo.value = null
    activeItemType.value = null
    activeItemIndex.value = null
    isInLongPressMode.value = false
  } catch (error) {
    console.error('删除失败:', error)
    showAlert('删除失败')
  }
}

function cancelDelete() {
  showDeleteConfirm.value = false
  deleteItemInfo.value = null
  activeItemType.value = null
  activeItemIndex.value = null
  isInLongPressMode.value = false
}

function cancelEdit() {
  isEditing.value = false
  activeItemType.value = null
  activeItemIndex.value = null
  isInLongPressMode.value = false
}

// 点击外部区域取消激活状态
function handleClickOutside(event: MouseEvent) {
  // 如果刚完成长按，则跳过这次点击处理，重置长按模式标志
  if (justFinishedLongPress.value) {
    isInLongPressMode.value = false
    return
  }

  if (
    cascadeContainer.value &&
    cascadeContainer.value.contains(event.target as Node)
  ) {
    // 点击在级联容器内部，不取消
    return
  }

  // 重置长按模式标志
  isInLongPressMode.value = false

  // 只有不在编辑状态时才取消激活
  if (!isEditing.value) {
    activeItemType.value = null
    activeItemIndex.value = null
    isEditing.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  window.addEventListener('resize', onWindowResize)
})

onUnmounted(() => {
  cancelLongPress()
  clearTimeout(longPressResetTimer)
  document.removeEventListener('click', handleClickOutside)
  window.removeEventListener('resize', onWindowResize)
})

// ==================== 错因分布环形图 ====================
const totalTags = computed(() => {
  return errorTagDistribution.value.reduce((sum, item) => sum + item.count, 0)
})

function calculatePercentage(count: number): number {
  if (totalTags.value === 0) return 0
  return Math.round((count / totalTags.value) * 100)
}

const donutSegments = computed(() => {
  const segments: {
    percent: number
    offset: number
    color: string
    linecap?: string
  }[] = []
  let currentOffset = 0
  const circumference = 251.33 // 2 * Math.PI * 40
  const gap = 0 // 扇区间隙

  errorTagDistribution.value.forEach((tag) => {
    const percent = tag.count / totalTags.value
    const rawLength = percent * circumference
    const drawnLength = Math.max(0.5, rawLength - gap)
    segments.push({
      percent: drawnLength,
      offset: currentOffset,
      color: tag.color,
      linecap: 'butt'
    })
    currentOffset -= rawLength
  })

  return segments
})

// ==================== 错因管理 ====================
function openManageModal() {
  console.log('=== 打开管理对话框 ===')
  console.log('所有错因标签:', errorTags.value)

  // 获取去重的错因标签（仅基于name），过滤掉软删除的标签
  const tagMap = new Map<
    string,
    { name: string; color: string; ids: string[] }
  >()

  errorTags.value.forEach((tag) => {
    const isDeleted = tag.name.startsWith('[已删除]')
    console.log(`标签 ${tag.name}, 是否删除: ${isDeleted}`)

    // 过滤掉软删除的标签
    if (isDeleted) {
      return
    }

    const key = tag.name
    if (!tagMap.has(key)) {
      tagMap.set(key, {
        name: tag.name,
        color: tag.color, // 使用第一个出现的颜色
        ids: []
      })
    }
    if (!tagMap.get(key)?.ids.includes(tag.id))
      tagMap.get(key)?.ids.push(tag.id)
  })

  const result = Array.from(tagMap.values())
  console.log('过滤后的标签列表:', result)

  manageErrorTags.value = result
  showManageModal.value = true
}

function closeManageModal() {
  showManageModal.value = false
  editingTagIndex.value = null
  editingTagName.value = ''
  editingTagColor.value = ''
}

function startEditTag(index: number) {
  editingTagIndex.value = index
  editingTagName.value = manageErrorTags.value[index].name
  editingTagColor.value = manageErrorTags.value[index].color
}

function cancelTagEdit() {
  editingTagIndex.value = null
  editingTagName.value = ''
  editingTagColor.value = ''
}

async function saveTagEdit(index: number) {
  if (!editingTagName.value.trim()) return

  try {
    const oldTag = manageErrorTags.value[index]
    const newName = editingTagName.value.trim()
    const newColor = editingTagColor.value || oldTag.color

    // 批量更新所有相同name的标签（包括颜色）
    const allSameNameTags = errorTags.value.filter(
      (t) => t.name === oldTag.name
    )

    for (const tag of new Map(
      allSameNameTags.map((tag) => [tag.id, tag])
    ).values()) {
      await updateTag({ id: tag.id, name: newName, color: newColor })
    }

    // 更新本地数据
    errorTags.value = errorTags.value.map((t) =>
      t.name === oldTag.name ? { ...t, name: newName, color: newColor } : t
    )

    // 更新manageErrorTags
    manageErrorTags.value[index].name = newName
    manageErrorTags.value[index].color = newColor

    cancelTagEdit()
  } catch (error) {
    console.error('更新错因标签失败:', error)
  }
}

function confirmDeleteTag(tag: UniqueErrorTag) {
  deleteTagName.value = tag.name
  showTagDeleteConfirm.value = true
}

async function executeDeleteTag() {
  const tag = manageErrorTags.value.find(
    (item) => item.name === deleteTagName.value
  )
  if (!tag) return
  try {
    for (const id of new Set(tag.ids)) {
      await deleteTag(id)
      // Publish each successful deletion; a later failure must not hide it.
      errorTags.value = errorTags.value.filter((item) => item.id !== id)
      tag.ids = tag.ids.filter((value) => value !== id)
    }
    manageErrorTags.value = manageErrorTags.value.filter((item) => item !== tag)
    showTagDeleteConfirm.value = false
  } catch (error) {
    showAlert('部分标签未能删除，可重试剩余项：' + String(error))
  }
}
</script>

<style scoped>
.profile-page {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: start;
  gap: 20px;
}

.page-heading,
.overview-cards,
.stats-row,
.loading-state,
.error-state {
  grid-column: 1 / -1;
}
.profile-page > .q-card {
  min-width: 0;
  box-shadow: none;
}
.chart-section {
  overflow: auto;
}
@media (max-width: 1023px) {
  .profile-page {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (min-width: 769px) {
  .profile-page {
    gap: 40px;
  }
}

/* 级联容器 */
.cascade-container {
  position: relative;
  margin-top: 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  animation: slideUp 0.3s ease;
}

/* 滚动包装器 */
.cascade-scroll-wrapper {
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
  scrollbar-color: var(--border-color) transparent;
}

.cascade-scroll-wrapper::-webkit-scrollbar {
  height: 6px;
}

.cascade-scroll-wrapper::-webkit-scrollbar-track {
  background: transparent;
}

.cascade-scroll-wrapper::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.cascade-scroll-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.cascade-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.cascade-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 20px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

/* 列包装器 */
.cascade-column-wrapper {
  display: flex;
  position: relative;
  overflow: visible;
  /* 小屏层叠参数 */
  --col-width: 160px;
  --col-gap: 80px;
}

/* 级联列 - 小屏：层叠重叠 */
.cascade-column {
  flex: 0 0 var(--col-width);
  width: var(--col-width);
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 1;
  background: var(--card-bg);
  max-height: 500px;
  overflow-y: auto;
}

.cascade-col-1 {
  border-left: none;
  z-index: 1;
}

.cascade-col-2,
.cascade-col-3,
.cascade-col-4 {
  margin-left: calc(-1 * (var(--col-width) - var(--col-gap)));
}

.cascade-col-2.show-column {
  z-index: 10;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.15);
}
.cascade-col-3.show-column {
  z-index: 20;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.15);
}
.cascade-col-4.show-column {
  z-index: 30;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.15);
}

.cascade-column.active-column {
  z-index: 100 !important;
}

/* 大屏（≥769px）：平铺展开，不再层叠 */
@media (min-width: 769px) {
  .cascade-column {
    flex: 0 0 clamp(140px, 18vw, 260px);
    width: clamp(140px, 18vw, 260px);
    position: static;
    z-index: auto;
    margin-left: 0 !important;
    box-shadow: none !important;
  }

  .cascade-column:first-child {
    padding-left: 8px;
  }

  .cascade-column:last-child {
    border-right: none;
  }

  .cascade-column.show-column {
    box-shadow: -2px 0 8px rgba(0, 0, 0, 0.08) !important;
  }
}

/* 列标题 */
.column-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

/* 列项 */
.column-item {
  padding: 10px 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-primary);
}

.column-item:hover {
  background: var(--primary-light);
}

.column-item.item-selected {
  background: var(--primary-color);
  color: white;
}

/* 项目点 */
.item-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* 项目文本 */
.item-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 空列项 */
.empty-column-item {
  padding: 20px 12px;
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
}

/* 科目分布可视化 */
.distribution-visual {
  margin-bottom: 20px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.stacked-bar {
  display: flex;
  height: 28px;
  border-radius: 14px;
  overflow: hidden;
  margin-bottom: 12px;
}

.bar-segment {
  transition: all 0.3s ease;
  min-width: 2px;
  cursor: pointer;
}

.bar-segment:hover {
  opacity: 0.85;
  transform: scaleY(1.1);
}

.bar-segment.segment-selected {
  box-shadow: inset 0 0 0 2px white;
  transform: scaleY(1.15);
  z-index: 10;
  position: relative;
}

.bar-total {
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
  margin-bottom: 12px;
  font-weight: 500;
}

.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: center;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  background: var(--card-bg);
  padding: 6px 12px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.legend-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.legend-item.legend-selected {
  box-shadow: 0 0 0 2px var(--primary-color);
  transform: translateY(-1px);
}

.legend-item {
  cursor: pointer;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  flex-shrink: 0;
}

.legend-label {
  color: var(--text-primary);
  font-weight: 500;
}

.legend-value {
  color: var(--text-secondary);
  font-size: 12px;
}

/* ========== 错因分布 ========== */
.error-tag-distribution {
  display: flex;
  gap: 24px;
  align-items: center;
  flex-wrap: wrap;
}

.donut-chart-wrapper {
  flex-shrink: 0;
  width: 220px;
  height: 220px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.donut-chart {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
  filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.1));
  overflow: visible;
}

.donut-segment {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  transform-origin: center;
  stroke-linejoin: round;
}

.donut-segment.segment-hovered {
  filter: brightness(1.1);
}

.donut-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  pointer-events: none;
  z-index: 10;
}

.donut-center-count {
  font-size: 28px;
  font-weight: 800;
  color: var(--primary-color);
  line-height: 1;
  margin-bottom: 4px;
}

.donut-center-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.error-tag-legend {
  flex: 1;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.error-tag-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border-radius: 10px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
}

.error-tag-item:hover {
  transform: translateX(6px);
  background: var(--primary-light);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.error-tag-item.item-highlighted {
  background: var(--primary-color);
  border-color: var(--primary-color);
  box-shadow: 0 4px 16px rgba(25, 118, 210, 0.3);
  transform: translateX(6px) scale(1.02);
}

.error-tag-item.item-highlighted .tag-name,
.error-tag-item.item-highlighted .tag-count {
  color: white;
}

.error-tag-item.item-highlighted .tag-percentage {
  background: white;
  color: var(--primary-color);
}

.error-tag-item.item-highlighted .tag-color-dot {
  border-color: white;
  transform: scale(1.2);
}

.tag-color-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
}

.tag-name {
  flex: 1;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  transition: color 0.25s ease;
}

.tag-count {
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 700;
  min-width: 30px;
  text-align: right;
  transition: color 0.25s ease;
}

.tag-percentage {
  background: var(--primary-color);
  color: white;
  font-size: 12px;
  font-weight: 700;
  padding: 3px 10px;
  border-radius: 12px;
  min-width: 45px;
  text-align: center;
  transition: all 0.25s ease;
}

/* ========== 加载 & 错误 ========== */
.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  gap: 16px;
  color: var(--text-secondary);
  font-size: 14px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-icon {
  width: 48px;
  height: 48px;
  color: #ef5350;
  opacity: 0.7;
}

.error-description {
  color: var(--text-secondary);
  font-size: 14px;
  text-align: center;
}

.retry-btn {
  padding: 8px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

/* ========== 概览卡片 ========== */
.overview-cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

@media (max-width: 599px) {
  .overview-cards {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.overview-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.06),
    0 1px 2px rgba(0, 0, 0, 0.04);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.overview-card:hover {
  transform: translateY(-2px);
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.08),
    0 2px 8px rgba(0, 0, 0, 0.06);
}

.overview-card:active {
  transform: scale(0.97);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.card-icon {
  font-size: 32px;
}
.card-content {
  flex: 1;
}
.card-value {
  font-size: 24px;
  font-weight: bold;
  color: var(--primary-color);
}
.card-value.due {
  color: #ef5350;
}
.card-value.memory {
  color: #66bb6a;
}
.card-value.new {
  color: #ab47bc;
}
.card-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

/* ========== SRS 详细指标 ========== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.stat-item {
  background: var(--card-bg);
  border-radius: 8px;
  padding: 12px 8px;
  text-align: center;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.stat-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.stat-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-sub {
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 2px;
  opacity: 0.7;
}

/* ========== 图表公用 ========== */
.chart-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

@media (min-width: 769px) {
  .chart-section {
    padding: 32px;
  }
}

.chart-section h3 {
  font-size: 16px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 0;
  color: var(--text-secondary);
  font-size: 14px;
}

/* ========== 科目分布 ========== */
.distribution-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stacked-bar {
  display: flex;
  height: 24px;
  border-radius: 12px;
  overflow: hidden;
}

.bar-segment {
  transition: width 0.3s;
  min-width: 2px;
}

.bar-total {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.legend {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  flex-shrink: 0;
}

.legend-label {
  flex: 1;
  color: var(--text-primary);
}
.legend-value {
  color: var(--text-secondary);
}

/* ========== 热力图 ========== */
.heatmap-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.heatmap-wrapper {
  overflow-x: auto;
}

.heatmap-table {
  border-collapse: collapse;
  width: 100%;
  min-width: 480px;
  font-size: 12px;
}

.heatmap-table th,
.heatmap-table td {
  text-align: center;
  padding: 6px 4px;
  border: 1px solid var(--border-color);
}

.row-header {
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-weight: 600;
  white-space: nowrap;
}

.col-header {
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 11px;
  width: 36px;
}

.total-col {
  font-size: 11px;
}

.row-label {
  text-align: left;
  padding: 6px 8px !important;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.heat-cell {
  transition: background 0.2s;
  color: var(--text-primary);
  font-size: 11px;
}

.row-total {
  font-weight: 600;
  color: var(--text-primary);
  background: var(--bg-secondary);
}

.heatmap-legend {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}

.legend-block {
  width: 16px;
  height: 12px;
  border-radius: 2px;
}

.difficulty-range {
  font-size: 11px;
  color: var(--text-secondary);
  text-align: right;
  margin-top: 4px;
}

/* 热力图点击详情 */
.cell-detail {
  margin-top: 12px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.8;
}

.cell-detail-row {
  color: var(--text-primary);
}

.cell-detail-row strong {
  color: var(--primary-color);
  font-size: 18px;
}

.heat-cell.selected {
  outline: 2px solid var(--primary-color);
  outline-offset: -1px;
}

/* ========== 复习间隔分布 ========== */
.interval-chart {
  padding: 12px 0;
}

.interval-bars {
  display: flex;
  justify-content: space-around;
  align-items: flex-end;
  height: 140px;
  gap: 6px;
}

.interval-bar-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  justify-content: flex-end;
}

.bar-fill {
  width: 70%;
  max-width: 40px;
  border-radius: 4px 4px 0 0;
  min-height: 4px;
  transition: height 0.3s;
}

.bar-value {
  font-size: 12px;
  color: var(--text-primary);
  margin-bottom: 4px;
  font-weight: 500;
}

.bar-label {
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 6px;
  text-align: center;
}

/* ========== 科目/来源管理样式 ========== */

/* 项目激活状态 */
.column-item.item-active {
  background: var(--bg-primary) !important;
  border-left: 3px solid var(--primary-color);
}

/* 编辑表单 */
.edit-form {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--bg-secondary);
}

.edit-form input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
  color: var(--text-primary);
  background: var(--bg-secondary);
}

.edit-form input:focus {
  border-color: var(--primary-color);
}

.edit-form input[type='color'] {
  padding: 2px;
  height: 30px;
  cursor: pointer;
}

.edit-buttons {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

.btn-save,
.btn-cancel {
  padding: 4px 10px;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-save {
  background: #66bb6a;
  color: white;
}

.btn-save:hover {
  background: #4caf50;
}

.btn-cancel {
  background: #bdbdbd;
  color: white;
}

.btn-cancel:hover {
  background: #9e9e9e;
}

/* 操作按钮 */
.item-actions {
  display: flex;
  gap: 6px;
  margin-left: auto;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.edit-btn {
  background: #e3f2fd;
  color: #1976d2;
}

.edit-btn:hover {
  background: #bbdefb;
  transform: scale(1.1);
}

.delete-btn {
  background: #ffebee;
  color: #ef5350;
}

.delete-btn:hover {
  background: #ffcdd2;
  transform: scale(1.1);
}

/* 删除确认弹窗 */
.delete-confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

.delete-confirm-modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 360px;
  width: 90%;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  animation: slideIn 0.2s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.delete-confirm-modal h3 {
  font-size: 18px;
  font-weight: 600;
  color: #757575;
  margin-bottom: 12px;
}

.delete-confirm-modal p {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.warning-text {
  color: #ef5350 !important;
  font-size: 13px !important;
  margin-top: 8px;
}

.modal-buttons {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
}

.btn-danger {
  padding: 8px 20px;
  background: #ef5350;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-danger:hover {
  background: #e53935;
  transform: translateY(-1px);
}

.btn-secondary {
  padding: 8px 20px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--border-color);
}

/* ========== 响应式 ========== */
@media (max-width: 480px) {
  .overview-cards {
    gap: 8px;
  }
  .card-value {
    font-size: 20px;
  }
  .legend {
    grid-template-columns: 1fr;
  }
  .bar-label {
    font-size: 9px;
  }
}

/* ========== 错因管理 ========== */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.manage-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.manage-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(25, 118, 210, 0.3);
}

.manage-icon {
  font-size: 16px;
}

/* 弹窗样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-container {
  background: var(--card-bg);
  border-radius: 16px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  animation: slideUp 0.3s ease;
}

.confirm-modal {
  max-width: 400px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
}

.modal-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.modal-close {
  width: 32px;
  height: 32px;
  border: none;
  background: var(--bg-secondary);
  border-radius: 8px;
  font-size: 20px;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--primary-light);
  color: var(--primary-color);
}

.modal-body {
  padding: 20px 24px;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 按钮样式 */
.btn {
  padding: 10px 24px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-secondary:hover {
  background: var(--border-color);
}

.btn-danger {
  background: #ef5350;
  color: white;
}

.btn-danger:hover {
  background: #e53935;
  box-shadow: 0 4px 12px rgba(229, 57, 53, 0.3);
}

/* 标签列表 */
.empty-list {
  padding: 40px 0;
  text-align: center;
}

.empty-text {
  color: var(--text-secondary);
  font-size: 14px;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.tag-item:hover {
  background: var(--primary-light);
}

.tag-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.tag-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  background: var(--primary-color);
}

.tag-name-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.tag-input {
  flex: 1;
  max-width: 200px;
  padding: 8px 12px;
  border: 2px solid var(--primary-color);
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  background: var(--bg-secondary);
  outline: none;
}

.tag-color-input {
  width: 36px;
  height: 36px;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  padding: 0;
  cursor: pointer;
  background: none;
}

.tag-color-input::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.tag-color-input::-webkit-color-swatch {
  border: none;
  border-radius: 5px;
}

.tag-actions {
  display: flex;
  gap: 8px;
}

.actions-group {
  display: flex;
  gap: 8px;
}

.action-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.edit-btn {
  background: #e3f2fd;
  color: #1976d2;
}

.edit-btn:hover {
  background: #bbdefb;
  transform: scale(1.05);
}

.delete-btn {
  background: #ffebee;
  color: #ef5350;
}

.delete-btn:hover {
  background: #ffcdd2;
  transform: scale(1.05);
}

.save-btn {
  background: #e8f5e9;
  color: #43a047;
}

.save-btn:hover {
  background: #c8e6c9;
  transform: scale(1.05);
}

.cancel-btn {
  background: #f5f5f5;
  color: #757575;
}

.cancel-btn:hover {
  background: #eeeeee;
  transform: scale(1.05);
}

/* 确认弹窗 */
.confirm-text {
  font-size: 16px;
  color: var(--text-primary);
  margin: 0 0 8px 0;
  text-align: center;
}

.confirm-hint {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
  text-align: center;
}
</style>
