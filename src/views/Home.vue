<template>
  <div class="home-page">
    <div class="header">
      <h1>智能错题本</h1>
      <p class="subtitle">让每一次错误，都成为进步的阶梯</p>
    </div>

    <!-- 轮播组件 -->
    <div class="carousel-section">
      <div
        class="arco-carousel arco-carousel-indicator-position-bottom"
        style="width: 100%; height: 200px; margin-bottom: 30px"
      >
        <div class="arco-carousel-slide arco-carousel-horizontal">
          <div
            class="arco-carousel-item"
            :class="{
              'arco-carousel-item-current': currentSlide === 0,
              'arco-carousel-item-next': currentSlide === 2,
              'arco-carousel-item-prev': currentSlide === 1
            }"
            style="
              transition-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              transition-duration: 500ms;
              animation-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              animation-duration: 500ms;
              background: rgb(54, 77, 121);
              color: white;
              text-align: center;
              line-height: 200px;
              font-size: 30px;
            "
          >
            欢迎使用智能错题本
          </div>
          <div
            class="arco-carousel-item"
            :class="{
              'arco-carousel-item-current': currentSlide === 1,
              'arco-carousel-item-next': currentSlide === 0,
              'arco-carousel-item-prev': currentSlide === 2
            }"
            style="
              transition-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              transition-duration: 500ms;
              animation-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              animation-duration: 500ms;
              background: rgb(0, 168, 84);
              color: white;
              text-align: center;
              line-height: 200px;
              font-size: 30px;
            "
          >
            好好学习，天天向上！
          </div>
          <div
            class="arco-carousel-item"
            :class="{
              'arco-carousel-item-current': currentSlide === 2,
              'arco-carousel-item-next': currentSlide === 1,
              'arco-carousel-item-prev': currentSlide === 0
            }"
            style="
              transition-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              transition-duration: 500ms;
              animation-timing-function: cubic-bezier(0.34, 0.69, 0.1, 1);
              animation-duration: 500ms;
              background: rgb(245, 34, 45);
              color: white;
              text-align: center;
              line-height: 200px;
              font-size: 30px;
            "
          >
            不要把梦想埋没！
          </div>
        </div>
        <div
          class="swiper-pagination swiper-pagination-clickable swiper-pagination-bullets"
        >
          <span
            data-index="0"
            class="swiper-pagination-bullet"
            :class="{ 'swiper-pagination-bullet-active': currentSlide === 0 }"
            tabindex="0"
            role="button"
            aria-label="Go to slide 1"
          ></span>
          <span
            data-index="1"
            class="swiper-pagination-bullet"
            :class="{ 'swiper-pagination-bullet-active': currentSlide === 1 }"
            tabindex="0"
            role="button"
            aria-label="Go to slide 2"
          ></span>
          <span
            data-index="2"
            class="swiper-pagination-bullet"
            :class="{ 'swiper-pagination-bullet-active': currentSlide === 2 }"
            tabindex="0"
            role="button"
            aria-label="Go to slide 3"
          ></span>
        </div>
        <div class="arco-carousel-arrow">
          <div class="arco-carousel-arrow-left">
            <svg
              viewBox="0 0 48 48"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              stroke="currentColor"
              class="arco-icon arco-icon-left"
              stroke-width="4"
              stroke-linecap="butt"
              stroke-linejoin="miter"
            >
              <path d="M32 8.4 16.444 23.956 32 39.513"></path>
            </svg>
          </div>
          <div class="arco-carousel-arrow-right">
            <svg
              viewBox="0 0 48 48"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              stroke="currentColor"
              class="arco-icon arco-icon-right"
              stroke-width="4"
              stroke-linecap="butt"
              stroke-linejoin="miter"
            >
              <path d="m16 39.513 15.556-15.557L16 8.4"></path>
            </svg>
          </div>
        </div>
      </div>
    </div>

    <div class="quick-actions">
      <h2>快捷操作</h2>
      <div class="action-grid">
        <button class="action-btn" @click="$router.push('/add')">
          <Icon name="plus" :size="24" class="btn-icon" />
          <span>添加错题</span>
        </button>
        <button v-if="hasDue" class="action-btn has-review" @click="$router.push('/review')">
          <Icon name="book-open" :size="24" class="btn-icon" />
          <span>开始复习</span>
        </button>
        <button v-else class="action-btn" @click="$router.push('/review')">
          <Icon name="book-open" :size="24" class="btn-icon" />
          <span>开始复习</span>
        </button>
        <button class="action-btn" @click="$router.push('/manage')">
          <Icon name="clipboard-list" :size="24" class="btn-icon" />
          <span>错题管理</span>
        </button>
        <button class="action-btn" @click="$router.push('/stats')">
          <Icon name="chart-column" :size="24" class="btn-icon" />
          <span>数据分析</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getDueCount } from '../apis';

// 轮播组件逻辑
const currentSlide = ref(0)
const slideCount = 3
let carouselInterval: number | null = null
let hasDue = ref(false)

const nextSlide = () => {
  currentSlide.value = (currentSlide.value + 1) % slideCount
  updateSlideClasses()
}

const prevSlide = () => {
  currentSlide.value = (currentSlide.value - 1 + slideCount) % slideCount
  updateSlideClasses()
}

const goToSlide = (index: number) => {
  currentSlide.value = index
  updateSlideClasses()
}

const updateSlideClasses = () => {
  const slides = document.querySelectorAll('.arco-carousel-item')
  slides.forEach((slide, index) => {
    slide.classList.remove(
      'arco-carousel-item-current',
      'arco-carousel-item-next',
      'arco-carousel-item-prev'
    )
    if (index === currentSlide.value) {
      slide.classList.add('arco-carousel-item-current')
    } else if (index === (currentSlide.value + 1) % slideCount) {
      slide.classList.add('arco-carousel-item-next')
    } else if (index === (currentSlide.value - 1 + slideCount) % slideCount) {
      slide.classList.add('arco-carousel-item-prev')
    }
  })
}

onMounted(() => {
  // 启动自动轮播
  carouselInterval = window.setInterval(nextSlide, 3000)

  // 添加指示器点击事件
  const indicators = document.querySelectorAll('.swiper-pagination-bullet')
  indicators.forEach((indicator, index) => {
    indicator.addEventListener('click', () => goToSlide(index))
  })

  // 添加箭头点击事件
  const leftArrow = document.querySelector('.arco-carousel-arrow-left')
  const rightArrow = document.querySelector('.arco-carousel-arrow-right')
  if (leftArrow) leftArrow.addEventListener('click', prevSlide)
  if (rightArrow) rightArrow.addEventListener('click', nextSlide)

  getDueCount().then(count => {
    console.log('due count:', count)
    hasDue.value = count > 0;
    // hasDue.value = true
  })
})

onUnmounted(() => {
  // 清除轮播定时器
  if (carouselInterval) {
    clearInterval(carouselInterval)
  }
})
</script>

<style scoped>
.home-page {
  padding: 20px;
  padding-bottom: 80px;
  background: var(--bg-primary);
  min-height: 100vh;
  margin: 0 auto;
}

.header {
  text-align: center;
  margin-bottom: 30px;
}

.header h1 {
  font-size: 28px;
  color: var(--primary-color);
  margin: 0 0 8px 0;
}

.subtitle {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.stats-cards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 30px;
}

.stat-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  font-size: 32px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: var(--primary-color);
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.quick-actions {
  margin-bottom: 30px;
}

.quick-actions h2 {
  font-size: 18px;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}

@media (max-width: 768px) {
  .action-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 480px) {
  .action-grid {
    grid-template-columns: 1fr;
  }
}

.action-btn {
  background: var(--card-bg);
  color: var(--text-primary);
  border: none;
  border-radius: 12px;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.action-btn.primary {
  background: var(--primary-color);
  color: white;
}

.action-btn:active {
  transform: scale(0.98);
}

.action-btn.has-review {
  background: linear-gradient(135deg, #ff9800 0%, #ff5722 100%);
  color: white;
  box-shadow: 0 4px 20px rgba(255, 87, 34, 0.4);
  animation: pulse-glow 0.8s ease-in-out infinite;
}

.action-btn.has-review .btn-icon {
  filter: brightness(1.1);
  transform: scale(1.1);
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: 0 4px 20px rgba(255, 87, 34, 0.4);
    transform: scale(1);
  }
  20% {
    box-shadow: 0 8px 40px rgba(255, 107, 62, 0.6);
    transform: scale(1.02);
  }
}

.btn-icon {
  font-size: 32px;
}

.action-btn span:last-child {
  font-size: 14px;
  font-weight: 500;
}

/* 轮播组件样式 */
.carousel-section {
  margin-bottom: 30px;
}

.arco-carousel {
  position: relative;
  overflow: hidden;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.arco-carousel-slide {
  position: relative;
  display: flex;
  height: 100%;
}

.arco-carousel-item {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  transition: all 0.5s cubic-bezier(0.34, 0.69, 0.1, 1);
  pointer-events: none;
}

.arco-carousel-item.arco-carousel-item-current {
  opacity: 1;
  pointer-events: auto;
}

/* Swiper Pagination */
.swiper-pagination {
  position: absolute;
  bottom: 16px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  gap: 8px;
  z-index: 10;
}

.swiper-pagination-bullet {
  width: 36px;
  height: 36px;
  background: url('../assets/carousel-indicator-inactive.png') no-repeat center;
  background-size: 100%;
  transition: all 0.5s;
  opacity: 0.5;
  transform: scale(0.85);
  border: none;
  outline: none;
}

.swiper-pagination-bullet-active {
  background: url('../assets/carousel-indicator.png') no-repeat center;
  background-size: 100%;
  opacity: 1;
  transform: scale(1);
}

.arco-carousel-arrow {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  transform: translateY(-50%);
  display: flex;
  justify-content: space-between;
  padding: 0 16px;
  z-index: 10;
}

.arco-carousel-arrow-left,
.arco-carousel-arrow-right {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s;
  color: white;
}

.arco-carousel-arrow-left:hover,
.arco-carousel-arrow-right:hover {
  background: rgba(0, 0, 0, 0.5);
}

.arco-icon {
  width: 20px;
  height: 20px;
}
</style>
