import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../../../src/middleware/error-handler';
import { slideService } from '../../../../../src/lib/services';
import type { CreateSlideBody, ReorderSlidesBody } from '../../../../../src/types/api';

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const data = await slideService.getPresentationData(slug);
    return NextResponse.json(data);
  } catch (error) {
    return handleError(error);
  }
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const body: CreateSlideBody = await request.json();
    const slide = await slideService.createSlide(slug, body.content, body.index);
    return NextResponse.json({ slide }, { status: 201 });
  } catch (error) {
    return handleError(error);
  }
}

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const body: ReorderSlidesBody = await request.json();
    const slides = await slideService.reorderSlides(slug, body.orderedSids);
    return NextResponse.json({ slides });
  } catch (error) {
    return handleError(error);
  }
}
