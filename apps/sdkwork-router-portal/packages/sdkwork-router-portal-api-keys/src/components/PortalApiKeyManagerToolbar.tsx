import { Plus, RefreshCw } from 'lucide-react';
import {
  Button,
  InlineButton,
  ToolbarInline,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';

export function PortalApiKeyManagerToolbar({
  onOpenCreate,
  onOpenUsage,
  onRefresh,
  onSearchChange,
  searchQuery,
}: {
  onOpenCreate: () => void;
  onOpenUsage?: () => void;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  searchQuery: string;
}) {
  const { t } = usePortalI18n();

  return (
    <section
      data-slot="portal-api-key-manager"
      className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
    >
      <ToolbarInline>
        <ToolbarSearchField
          label={t('Search API keys')}
          value={searchQuery}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder={t('Search API keys')}
          className="min-w-[15rem] flex-[0_1_20rem]"
        />
        <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
          <Button type="button" onClick={onOpenCreate}>
            <Plus className="h-4 w-4" />
            {t('Create API key')}
          </Button>

          <InlineButton onClick={onRefresh} tone="secondary">
            <RefreshCw className="h-4 w-4" />
            {t('Refresh')}
          </InlineButton>

          {onOpenUsage ? (
            <InlineButton onClick={onOpenUsage} tone="secondary">
              {t('Open usage')}
            </InlineButton>
          ) : null}
        </div>
      </ToolbarInline>
    </section>
  );
}
