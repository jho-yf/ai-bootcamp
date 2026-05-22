import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../../../../src/middleware/error-handler';
import { styleService } from '../../../../../../src/lib/services';
import type { GenerateStyleBody, SelectStyleBody } from '../../../../../../src/types/api';

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const body: GenerateStyleBody = await request.json();
    const result = await styleService.generateCandidates(slug, body.prompt);
    return NextResponse.json(result);
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
    const body: SelectStyleBody = await request.json();
    const style = await styleService.selectStyle(slug, body.referenceImage);
    return NextResponse.json({ style });
  } catch (error) {
    return handleError(error);
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    await styleService.cleanupCandidates(slug);
    return new NextResponse(null, { status: 204 });
  } catch (error) {
    return handleError(error);
  }
}
