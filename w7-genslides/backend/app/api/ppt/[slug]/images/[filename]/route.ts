import { NextRequest, NextResponse } from 'next/server';
import { imageRepo } from '../../../../../../src/lib/services';

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string; filename: string }> },
) {
  const { slug, filename } = await params;
  const buffer = await imageRepo.read(slug, filename);

  if (!buffer) {
    return NextResponse.json(
      { error: { code: 'NOT_FOUND', message: 'Image not found' } },
      { status: 404 },
    );
  }

  return new NextResponse(new Uint8Array(buffer), {
    headers: {
      'Content-Type': 'image/jpeg',
      'Cache-Control': 'max-age=31536000, immutable',
    },
  });
}
