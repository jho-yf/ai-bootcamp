<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { Slide } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  slides: Slide[]
  slug: string
  visible: boolean
  startSid?: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

const presentationStore = usePresentationStore()

const isPlaying = ref(false)
const playingIndex = ref(0)
const showReplayHint = ref(false)

const slidesWithImages = computed(() =>
  props.slides.filter((s) => s.images.length > 0),
)

const currentSlide = computed(() => slidesWithImages.value[playingIndex.value] ?? null)

const currentImageUrl = computed(() => {
  const slide = currentSlide.value
  if (!slide) return null
  const image = slide.images[slide.activeImageIndex]
  if (!image) return null
  return presentationStore.imageUrl(props.slug, image.filename)
})

function startPlayback() {
  if (slidesWithImages.value.length === 0) return
  isPlaying.value = true
  showReplayHint.value = false
  document.documentElement.requestFullscreen?.()
  if (props.startSid) {
    const idx = slidesWithImages.value.findIndex((s) => s.sid === props.startSid)
    playingIndex.value = idx >= 0 ? idx : 0
  } else {
    playingIndex.value = 0
  }
}

function goTo(idx: number) {
  showReplayHint.value = false
  playingIndex.value = idx
}

function showPrev() {
  const idx = (playingIndex.value - 1 + slidesWithImages.value.length) % slidesWithImages.value.length
  goTo(idx)
}

function showNext() {
  if (showReplayHint.value) {
    goTo(0)
    return
  }
  if (playingIndex.value >= slidesWithImages.value.length - 1) {
    showReplayHint.value = true
  } else {
    goTo(playingIndex.value + 1)
  }
}

function stopPlayback() {
  isPlaying.value = false
  showReplayHint.value = false
  emit('close')
}

function onFullscreenChange() {
  if (!document.fullscreenElement && isPlaying.value) {
    stopPlayback()
  }
}

function onKeydown(e: KeyboardEvent) {
  if (!isPlaying.value) return
  if (e.key === 'Escape') {
    stopPlayback()
  } else if (e.key === 'ArrowLeft') {
    showPrev()
  } else if (e.key === 'ArrowRight') {
    showNext()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
  document.addEventListener('fullscreenchange', onFullscreenChange)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
  document.removeEventListener('fullscreenchange', onFullscreenChange)
})

watch(
  () => props.visible,
  (val) => {
    if (val) {
      startPlayback()
    } else {
      isPlaying.value = false
    }
  },
)
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isPlaying"
        class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/95 bg-[radial-gradient(circle_at_center,#1f2937_0%,#020617_70%)] p-4"
        @click="stopPlayback"
      >
        <div class="relative h-full w-full" @click.stop>
          <Transition mode="out-in">
            <img
              v-if="currentImageUrl"
              :key="playingIndex"
              :src="currentImageUrl"
              class="h-full w-full object-contain drop-shadow-2xl"
              enter-active-class="transition duration-300 ease-out"
              enter-from-class="opacity-0 scale-95"
              enter-to-class="opacity-100 scale-100"
              leave-active-class="transition duration-200 ease-in"
              leave-from-class="opacity-100 scale-100"
              leave-to-class="opacity-0 scale-105"
            />
          </Transition>
        </div>

        <div class="absolute bottom-8 left-1/2 flex -translate-x-1/2 items-center gap-3 rounded-full bg-black/30 px-4 py-2 backdrop-blur">
          <button
            v-for="(_, idx) in slidesWithImages"
            :key="idx"
            class="h-2 w-2 rounded-full transition-all duration-300"
            :class="idx === playingIndex ? 'scale-125 bg-white' : 'bg-white/40 hover:bg-white/60'"
            @click.stop="goTo(idx)"
          />
          <span class="text-xs text-white/60">{{ playingIndex + 1 }} / {{ slidesWithImages.length }}</span>
        </div>

        <div class="absolute top-4 right-4 text-white/40 text-sm">
          ← → 切换，ESC 退出
        </div>

        <Transition
          enter-active-class="transition duration-300 ease-out"
          enter-from-class="translate-y-4 opacity-0"
          enter-to-class="translate-y-0 opacity-100"
          leave-active-class="transition duration-200 ease-in"
          leave-from-class="translate-y-0 opacity-100"
          leave-to-class="translate-y-4 opacity-0"
        >
          <div
            v-if="showReplayHint"
            class="absolute bottom-20 left-1/2 -translate-x-1/2 flex items-center gap-2 rounded-full bg-black/50 px-5 py-2.5 text-sm font-medium text-white backdrop-blur"
          >
            已是最后一张，再按一次 → 回到第一张，ESC 退出
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
