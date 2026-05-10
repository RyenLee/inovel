<template>
  <div class="category-tabs-container" ref="containerRef">
    <div
      class="scroll-shadow-left"
      :class="{ visible: showLeftShadow }"
      @click="scrollLeft"
    ></div>
    <div
      class="scroll-shadow-right"
      :class="{ visible: showRightShadow }"
      @click="scrollRight"
    ></div>

    <div
      class="category-tabs-wrapper"
      ref="wrapperRef"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
      @mousedown="handleMouseDown"
    >
      <div
        class="category-tabs-track"
        ref="trackRef"
        :style="{ transform: `translateX(${scrollOffset}px)` }"
      >
        <button
          v-for="category in categories"
          :key="category.name"
          :class="[
            'category-tab',
            { active: activeCategory === category.name },
          ]"
          @click="selectCategory(category.name)"
          :title="category.description"
        >
          <span class="tab-label">{{ category.label }}</span>
          <span
            v-if="activeCategory === category.name"
            class="tab-indicator"
          ></span>
        </button>
      </div>
    </div>

    <div class="scroll-hint" v-if="hasScroll">
      <span class="hint-text">{{ t('common.scrollHint') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

interface Category {
  name: string;
  label: string;
  description?: string;
}

const props = defineProps<{
  categories: Category[];
  activeCategory: string;
}>();

const emit = defineEmits<{
  (e: "update:activeCategory", value: string): void;
}>();

const containerRef = ref<HTMLElement | null>(null);
const wrapperRef = ref<HTMLElement | null>(null);
const trackRef = ref<HTMLElement | null>(null);
const scrollOffset = ref(0);

// Touch state
const touchStartX = ref(0);
const touchCurrentX = ref(0);
const lastTouchTime = ref(0);
const velocity = ref(0);
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartOffset = ref(0);
const animationFrame = ref<number | null>(null);
const animateOffset = ref(0);

const canScrollLeft = computed(() => scrollOffset.value < 0);
const canScrollRight = computed(() => {
  if (!trackRef.value || !wrapperRef.value) return false;
  const trackWidth = trackRef.value.scrollWidth;
  const wrapperWidth = wrapperRef.value.clientWidth;
  return scrollOffset.value > -(trackWidth - wrapperWidth);
});

const showLeftShadow = computed(
  () => canScrollLeft.value && Math.abs(scrollOffset.value) > 10
);

const showRightShadow = computed(
  () =>
    canScrollRight.value &&
    (trackRef.value?.scrollWidth || 0) >
      (wrapperRef.value?.clientWidth || 0) + 10
);

const hasScroll = computed(() => {
  if (!trackRef.value || !wrapperRef.value) return false;
  return trackRef.value.scrollWidth > wrapperRef.value.clientWidth;
});

const selectCategory = (name: string) => {
  emit("update:activeCategory", name);
};

const updateScrollShadows = () => {
  nextTick(() => {
    // Shadows are handled by computed properties
  });
};

const clampOffset = (offset: number): number => {
  if (!trackRef.value || !wrapperRef.value) return offset;
  const trackWidth = trackRef.value.scrollWidth;
  const wrapperWidth = wrapperRef.value.clientWidth;
  const maxOffset = trackWidth - wrapperWidth;
  return Math.max(-maxOffset, Math.min(0, offset));
};

const handleTouchStart = (e: TouchEvent) => {
  if (animationFrame.value) {
    cancelAnimationFrame(animationFrame.value);
    animationFrame.value = null;
  }
  touchStartX.value = e.touches[0].clientX;
  touchCurrentX.value = touchStartX.value;
  lastTouchTime.value = Date.now();
  velocity.value = 0;
};

const handleTouchMove = (e: TouchEvent) => {
  e.preventDefault();
  const currentX = e.touches[0].clientX;
  const deltaX = currentX - touchCurrentX.value;
  const now = Date.now();
  const timeDelta = now - lastTouchTime.value;

  if (timeDelta > 0) {
    velocity.value = deltaX / timeDelta;
  }

  touchCurrentX.value = currentX;
  lastTouchTime.value = now;

  animateOffset.value = clampOffset(scrollOffset.value + deltaX);
  scrollOffset.value = animateOffset.value;
  updateScrollShadows();
};

const handleTouchEnd = () => {
  const friction = 0.95;
  const minVelocity = 0.5;

  const animate = () => {
    velocity.value *= friction;

    if (Math.abs(velocity.value) > minVelocity) {
      animateOffset.value = clampOffset(
        scrollOffset.value + velocity.value * 16
      );
      scrollOffset.value = animateOffset.value;
      animationFrame.value = requestAnimationFrame(animate);
    } else {
      scrollOffset.value = clampOffset(scrollOffset.value);
      animationFrame.value = null;
    }

    updateScrollShadows();
  };

  animate();
};

const handleMouseDown = (e: MouseEvent) => {
  if (e.button !== 0) return;

  isDragging.value = true;
  dragStartX.value = e.clientX;
  dragStartOffset.value = scrollOffset.value;

  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return;

  const deltaX = e.clientX - dragStartX.value;
  scrollOffset.value = clampOffset(dragStartOffset.value + deltaX);
  updateScrollShadows();
};

const handleMouseUp = () => {
  isDragging.value = false;
  document.removeEventListener("mousemove", handleMouseMove);
  document.removeEventListener("mouseup", handleMouseUp);
};

const scrollLeft = () => {
  if (!wrapperRef.value) return;
  const scrollAmount = wrapperRef.value.clientWidth * 0.7;
  scrollOffset.value = clampOffset(scrollOffset.value + scrollAmount);
  updateScrollShadows();
};

const scrollRight = () => {
  if (!wrapperRef.value) return;
  const scrollAmount = wrapperRef.value.clientWidth * 0.7;
  scrollOffset.value = clampOffset(scrollOffset.value - scrollAmount);
  updateScrollShadows();
};

const handleResize = () => {
  scrollOffset.value = clampOffset(scrollOffset.value);
  updateScrollShadows();
};

watch(
  () => props.categories,
  () => {
    nextTick(() => {
      scrollOffset.value = clampOffset(scrollOffset.value);
      updateScrollShadows();
    });
  },
  { deep: true }
);

watch(
  () => props.activeCategory,
  () => {
    nextTick(() => {
      const wrapper = wrapperRef.value;
      if (!wrapper) return;
      
      const activeTab = wrapper.querySelector(".category-tab.active");

      if (activeTab) {
        const wrapperRect = wrapper.getBoundingClientRect();
        const tabRect = activeTab.getBoundingClientRect();
        const tabLeft = tabRect.left - wrapperRect.left;
        const tabRight = tabRect.right - wrapperRect.left;

        if (tabLeft < 0) {
          scrollOffset.value = clampOffset(scrollOffset.value + tabLeft);
        } else if (tabRight > wrapperRect.width) {
          scrollOffset.value = clampOffset(
            scrollOffset.value - (tabRight - wrapperRect.width)
          );
        }

        updateScrollShadows();
      }
    });
  }
);

onMounted(() => {
  nextTick(() => {
    updateScrollShadows();
  });
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  if (animationFrame.value) {
    cancelAnimationFrame(animationFrame.value);
  }
  document.removeEventListener("mousemove", handleMouseMove);
  document.removeEventListener("mouseup", handleMouseUp);
  window.removeEventListener("resize", handleResize);
});
</script>

<style scoped>
.category-tabs-container {
  position: relative;
  width: 100%;
  overflow: hidden;
  background: linear-gradient(145deg, #ffffff 0%, #f8fafc 100%);
  border: 2px solid transparent;
  border-radius: 20px;
  box-shadow: 0 4px 20px rgba(79, 70, 229, 0.08), 0 1px 3px rgba(0, 0, 0, 0.05),
    inset 0 0 0 1px rgba(79, 70, 229, 0.1);
  padding: 4px;
  background-clip: padding-box;
}

.scroll-shadow-left,
.scroll-shadow-right {
  position: absolute;
  top: 4px;
  bottom: 4px;
  width: 56px;
  pointer-events: none;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.3s ease;
  border-radius: 16px;
}

.scroll-shadow-left.visible,
.scroll-shadow-right.visible {
  opacity: 1;
  pointer-events: auto;
  cursor: pointer;
}

.scroll-shadow-left {
  left: 4px;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0.98) 0%,
    rgba(255, 255, 255, 0.8) 50%,
    transparent 100%
  );
}

.scroll-shadow-right {
  right: 4px;
  background: linear-gradient(
    -90deg,
    rgba(255, 255, 255, 0.98) 0%,
    rgba(255, 255, 255, 0.8) 50%,
    transparent 100%
  );
}

.category-tabs-wrapper {
  overflow: hidden;
  cursor: grab;
  user-select: none;
  touch-action: pan-x;
}

.category-tabs-wrapper:active {
  cursor: grabbing;
}

.category-tabs-track {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  transition: transform 0.1s ease-out;
}

.category-tab {
  position: relative;
  display: inline-flex;
  align-items: center;
  padding: 12px 24px;
  margin: 4px 0;
  border: none;
  background: transparent;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  flex-shrink: 0;
}

.category-tab:hover {
  background: rgba(79, 70, 229, 0.08);
  color: #4f46e5;
  transform: translateY(-1px);
}

.category-tab.active {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
  color: #ffffff;
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.35);
  transform: translateY(-1px);
}

.tab-label {
  position: relative;
  z-index: 1;
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 24px;
  height: 3px;
  background: rgba(255, 255, 255, 0.4);
  border-radius: 2px;
}

.scroll-hint {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.hint-text {
  font-size: 12px;
  color: #9ca3af;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.5;
  }
  50% {
    opacity: 1;
  }
}

@media (max-width: 768px) {
  .category-tabs-track {
    padding: 6px 12px;
    gap: 6px;
  }

  .category-tab {
    padding: 10px 18px;
    font-size: 13px;
    margin: 2px 0;
  }

  .scroll-shadow-left,
  .scroll-shadow-right {
    width: 32px;
  }
}

@media (max-width: 480px) {
  .category-tabs-track {
    padding: 4px 8px;
    gap: 4px;
  }

  .category-tab {
    padding: 8px 14px;
    font-size: 12px;
    border-radius: 8px;
  }

  .scroll-shadow-left,
  .scroll-shadow-right {
    width: 24px;
  }
}

:deep(.dark) .category-tabs-container {
  background: linear-gradient(145deg, #1f2937 0%, #111827 100%);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3), 0 1px 3px rgba(0, 0, 0, 0.2),
    inset 0 0 0 1px rgba(79, 70, 229, 0.2);
}

:deep(.dark) .scroll-shadow-left {
  background: linear-gradient(
    90deg,
    rgba(31, 41, 55, 0.98) 0%,
    rgba(31, 41, 55, 0.8) 50%,
    transparent 100%
  );
}

:deep(.dark) .scroll-shadow-right {
  background: linear-gradient(
    -90deg,
    rgba(31, 41, 55, 0.98) 0%,
    rgba(31, 41, 55, 0.8) 50%,
    transparent 100%
  );
}

:deep(.dark) .category-tab {
  color: #9ca3af;
}

:deep(.dark) .category-tab:hover {
  background: rgba(79, 70, 229, 0.15);
}

:deep(.dark) .category-tab.active {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
}

:deep(.dark) .hint-text {
  color: #6b7280;
}
</style>
