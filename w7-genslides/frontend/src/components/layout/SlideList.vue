<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import type { Slide } from '../../types/slide'
import SlideCard from './SlideCard.vue'

const props = defineProps<{
  slides: Slide[]
  selectedSid: string | null
  generatingSids: Set<string>
}>()

const emit = defineEmits<{
  select: [sid: string]
  add: []
  delete: [sid: string]
  edit: [sid: string]
  view: [sid: string]
  create: [sid: string]
  reorder: [orderedSids: string[]]
}>()

let dragIndex: number | null = null
const scrollRef = ref<HTMLElement | null>(null)
const showScrollHint = ref(false)
const atBottom = ref(false)

function checkScroll() {
  const el = scrollRef.value
  if (!el) return
  const hasMore = el.scrollHeight > el.clientHeight
  const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
  showScrollHint.value = hasMore && distFromBottom > 30
  atBottom.value = hasMore && distFromBottom <= 30
}

onMounted(() => {
  nextTick(checkScroll)
  scrollRef.value?.addEventListener('scroll', checkScroll)
})

onUnmounted(() => {
  scrollRef.value?.removeEventListener('scroll', checkScroll)
})

watch(() => props.slides.length, () => nextTick(checkScroll))

watch(() => props.selectedSid, async () => {
  await nextTick()
  const el = scrollRef.value?.querySelector('[data-selected="true"]')
  el?.scrollIntoView({ block: 'nearest' })
})

function onDragStart(index: number) {
  dragIndex = index
}

function onDragOver(e: DragEvent, index: number) {
  e.preventDefault()
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  const midY = rect.top + rect.height / 2
  if (e.clientY < midY) {
    el.style.borderTop = '2px solid #3b82f6'
    el.style.borderBottom = ''
  } else {
    el.style.borderBottom = '2px solid #3b82f6'
    el.style.borderTop = ''
  }
}

function onDragLeave(e: DragEvent) {
  const el = e.currentTarget as HTMLElement
  el.style.borderTop = ''
  el.style.borderBottom = ''
}

function onDrop(e: DragEvent, dropIndex: number, slides: Slide[]) {
  const el = e.currentTarget as HTMLElement
  el.style.borderTop = ''
  el.style.borderBottom = ''

  if (dragIndex === null || dragIndex === dropIndex) return

  const reordered = [...slides]
  const [moved] = reordered.splice(dragIndex, 1)
  reordered.splice(dropIndex, 0, moved)
  emit(
    'reorder',
    reordered.map((s) => s.sid),
  )
  dragIndex = null
}
</script>

<template>
  <div class="flex items-center justify-between px-3 py-2">
    <span class="text-xs font-medium text-gray-500">Slides</span>
  </div>

  <div ref="scrollRef" class="scrollbar-hidden relative flex-1 space-y-2 overflow-y-auto px-3">
    <SlideCard
      v-for="(slide, index) in slides"
      :key="slide.sid"
      :slide="slide"
      :index="index"
      :is-selected="slide.sid === selectedSid"
      :is-generating="generatingSids.has(slide.sid)"
      draggable="true"
      @select="emit('select', slide.sid)"
      @delete="emit('delete', slide.sid)"
      @view="emit('view', slide.sid)"
      @create="emit('create', slide.sid)"
      @dragstart="onDragStart(index)"
      @dragover="(e: DragEvent) => onDragOver(e, index)"
      @dragleave="onDragLeave"
      @drop="(e: DragEvent) => onDrop(e, index, slides)"
    />

    <div
      v-if="showScrollHint"
      class="pointer-events-none sticky bottom-0 h-8 bg-gradient-to-t from-gray-50 to-transparent"
    />
    <div
      v-else-if="atBottom"
      class="py-2 text-center text-xs text-gray-300"
    >
      — 没有更多 —
    </div>
  </div>

  <div class="shrink-0 px-3 pt-2 pb-3">
    <button
      class="flex w-full items-center justify-center gap-1 rounded-lg border-2 border-dashed border-gray-300 py-3 text-sm text-gray-400 transition hover:border-blue-400 hover:text-blue-500"
      @click="emit('add')"
    >
      + 新建 Slide
    </button>
  </div>
</template>
