import type { DocumentKind } from "../types";

export type CloseCandidate = {
  id: string;
  kind: DocumentKind;
};

export type ClosePreparation =
  | { status: "ready"; commit?: () => Promise<boolean | void> | boolean | void }
  | { status: "cancelled" };

type ReadyClosePreparation = Extract<ClosePreparation, { status: "ready" }>;

export type CloseCoordinatorContext = {
  prepare: (candidate: CloseCandidate) => Promise<ClosePreparation>;
  release: (candidate: CloseCandidate) => Promise<void>;
  recordClosed?: (candidate: CloseCandidate) => Promise<void>;
};

export type CloseTransactionResult = {
  completed: boolean;
  cancelledAtId?: string;
};

export async function runCloseTransaction(
  candidates: CloseCandidate[],
  context: CloseCoordinatorContext,
): Promise<CloseTransactionResult> {
  if (candidates.length === 0) return { completed: true };

  const prepared: Array<{ candidate: CloseCandidate; preparation: ReadyClosePreparation }> = [];
  for (const candidate of candidates) {
    const preparation = await context.prepare(candidate);
    if (preparation.status === "cancelled") {
      return { completed: false, cancelledAtId: candidate.id };
    }
    prepared.push({ candidate, preparation });
  }

  for (const { candidate, preparation } of prepared) {
    const committed = await preparation.commit?.();
    if (committed === false) {
      return { completed: false, cancelledAtId: candidate.id };
    }
  }

  for (const { candidate } of prepared) {
    await context.release(candidate);
    await context.recordClosed?.(candidate);
  }
  return { completed: true };
}
