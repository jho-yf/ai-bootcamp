import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../../../../src/middleware/error-handler';
import { slideService } from '../../../../../../src/lib/services';
import type { UpdateSlideBody } from '../../../../../../src/types/api';

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string; sid: string }> },
) {
  try {
    const { slug, sid } = await params;
    const body: UpdateSlideBody = await request.json();
    const result = await slideService.updateSlide(slug, sid, body);
    return NextResponse.json(result);
  } catch (error) {
    return handleError(error);
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string; sid: string }> },
) {
  try {
    const { slug, sid } = await params;
    await slideService.deleteSlide(slug, sid);
    return new NextResponse(null, { status: 204 });
  } catch (error) {
    return handleError(error);
  }
}
