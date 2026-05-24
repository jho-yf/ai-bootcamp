import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../../../../../../src/middleware/error-handler';
import { slideService } from '../../../../../../../../src/lib/services';

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string; sid: string; index: string }> },
) {
  try {
    const { slug, sid, index } = await params;
    const imageIndex = Number(index);
    const slide = await slideService.deleteSlideImage(slug, sid, imageIndex);
    return NextResponse.json({ slide });
  } catch (error) {
    return handleError(error);
  }
}
