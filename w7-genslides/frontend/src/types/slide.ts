export interface SlideImage {
  filename: string
  textHash: string
  createdAt: string
  cost: number
}

export interface Slide {
  sid: string
  content: string
  images: SlideImage[]
  activeImageIndex: number
}
