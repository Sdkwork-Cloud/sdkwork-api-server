import { useState } from 'react';
import type { FormEvent } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  FormField,
  InlineButton,
  Input,
  MetricCard,
  Pill,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { UserProfileFacts } from '../components';
import { changePortalPassword } from '../repository';
import { buildPortalUserViewModel, passwordsMatch } from '../services';
import type { PortalUserPageProps } from '../types';

export function PortalUserPage({ workspace, onNavigate }: PortalUserPageProps) {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [status, setStatus] = useState(
    'Keep personal identity, password rotation, and self-service recovery inside the user boundary.',
  );
  const [submitting, setSubmitting] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  const viewModel = buildPortalUserViewModel(workspace, newPassword, confirmPassword);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!passwordsMatch(newPassword, confirmPassword)) {
      setStatus('New password confirmation does not match.');
      return;
    }
    if (!viewModel.can_submit_password) {
      setStatus('Password rotation does not yet satisfy the visible user security policy.');
      return;
    }

    setSubmitting(true);
    setStatus('Updating personal password...');

    try {
      await changePortalPassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setStatus('Password updated. Use the new password the next time you sign in.');
      setDialogOpen(false);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <div className="portalx-status-row">
        <Pill tone={workspace?.user.active ? 'positive' : 'warning'}>{workspace?.user.active ? 'Access active' : 'Access review'}</Pill>
        <span className="portal-shell-status-copy text-sm">{status}</span>
        <InlineButton onClick={() => onNavigate('account')} tone="secondary">
          Open account
        </InlineButton>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <button className="inline-flex h-10 items-center justify-center rounded-2xl bg-primary-600 px-4 text-sm font-medium text-white shadow-[0_16px_30px_rgb(var(--portal-accent-rgb)_/_0.22)] transition hover:bg-primary-500" type="button">
              Password rotation
            </button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Password rotation</DialogTitle>
              <DialogDescription>
                Update personal credentials in a focused dialog instead of keeping the full password form expanded on the page.
              </DialogDescription>
            </DialogHeader>
            <form className="grid gap-4" onSubmit={handleSubmit}>
              <FormField label="Current password">
                <Input
                  autoComplete="current-password"
                  onChange={(event) => setCurrentPassword(event.target.value)}
                  required
                  type="password"
                  value={currentPassword}
                />
              </FormField>
              <FormField label="New password">
                <Input
                  autoComplete="new-password"
                  onChange={(event) => setNewPassword(event.target.value)}
                  required
                  type="password"
                  value={newPassword}
                />
              </FormField>
              <FormField label="Confirm new password">
                <Input
                  autoComplete="new-password"
                  onChange={(event) => setConfirmPassword(event.target.value)}
                  required
                  type="password"
                  value={confirmPassword}
                />
              </FormField>

              <div className="portal-shell-info-card grid gap-3">
                {viewModel.password_policy.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.id}>
                    <span className="portal-shell-info-copy text-sm">{item.label}</span>
                    <Pill tone={item.met ? 'positive' : 'warning'}>{item.met ? 'Met' : 'Pending'}</Pill>
                  </div>
                ))}
              </div>

              <DialogFooter>
                <InlineButton onClick={() => setDialogOpen(false)} tone="ghost" type="button">
                  Cancel
                </InlineButton>
                <InlineButton tone="primary" type="submit">
                  {submitting ? 'Saving...' : 'Update password'}
                </InlineButton>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {viewModel.profile_facts.map((item) => (
          <MetricCard detail={item.detail} key={item.id} label={item.title} value={item.value} />
        ))}
      </div>

      <Tabs defaultValue="profile">
        <TabsList>
          <TabsTrigger value="profile">Profile</TabsTrigger>
          <TabsTrigger value="security">Security center</TabsTrigger>
          <TabsTrigger value="recovery">Recovery</TabsTrigger>
        </TabsList>

        <TabsContent value="profile">
          <Surface detail="The signed-in person and the workspace boundary they currently control." title="Profile facts">
            <div className="grid gap-4 lg:grid-cols-2">
              <div className="grid gap-3">
                {viewModel.profile_facts.map((item) => (
                  <article className="portal-shell-info-card" key={item.id}>
                    <strong className="portal-shell-info-title">{item.title}</strong>
                    <p className="mt-2 text-lg font-semibold text-[var(--portal-text-primary)]">{item.value}</p>
                    <p className="portal-shell-info-copy mt-2 text-sm">{item.detail}</p>
                  </article>
                ))}
              </div>
              <UserProfileFacts workspace={workspace} />
            </div>
          </Surface>
        </TabsContent>

        <TabsContent value="security">
          <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
            <Surface
              actions={
                <InlineButton onClick={() => setDialogOpen(true)} tone="primary">
                  Open password dialog
                </InlineButton>
              }
              detail="A clear checklist keeps personal security work explicit before the next routing, key, or financial action."
              title="Personal security checklist"
            >
              <div className="grid gap-3">
                {viewModel.personal_security_checklist.map((item) => (
                  <article className="portal-shell-info-card" key={item.id}>
                    <div className="flex items-center justify-between gap-3">
                      <strong className="portal-shell-info-title">{item.title}</strong>
                      <Pill tone={item.complete ? 'positive' : 'warning'}>
                        {item.complete ? 'Ready' : 'Needs action'}
                      </Pill>
                    </div>
                    <p className="portal-shell-info-copy mt-2 text-sm">{item.detail}</p>
                  </article>
                ))}
              </div>
            </Surface>

            <Surface
              detail="Password requirements remain visible even when the edit form is collapsed into a dialog."
              title="Password policy"
            >
              <div className="grid gap-3">
                {viewModel.password_policy.map((item) => (
                  <article className="portal-shell-info-card" key={item.id}>
                    <div className="flex items-center justify-between gap-3">
                      <strong className="portal-shell-info-title">{item.label}</strong>
                      <Pill tone={item.met ? 'positive' : 'warning'}>{item.met ? 'Met' : 'Pending'}</Pill>
                    </div>
                  </article>
                ))}
              </div>
            </Surface>
          </div>
        </TabsContent>

        <TabsContent value="recovery">
          <div className="grid gap-6 xl:grid-cols-2">
            <Surface
              detail="User recovery should guide the signed-in person back into productive work without confusing profile trust with money posture."
              title="Recovery signals"
            >
              <div className="grid gap-3">
                {viewModel.recovery_signals.map((item) => (
                  <article className="portal-shell-info-card" key={item.id}>
                    <strong className="portal-shell-info-title">{item.title}</strong>
                    <p className="portal-shell-info-copy mt-2 text-sm">{item.detail}</p>
                  </article>
                ))}
              </div>
            </Surface>

            <Surface
              detail="User work should reconnect with routing, usage, and financial review instead of ending in a dead settings screen."
              title="Return to command center"
            >
              <div className="grid gap-3">
                <article className="portal-shell-info-card">
                  <strong className="portal-shell-info-title">Return to the workspace pulse</strong>
                  <p className="portal-shell-info-copy mt-2 text-sm">After a personal trust update, confirm that the command center still reflects a healthy operating mode.</p>
                  <div className="mt-4">
                    <InlineButton onClick={() => onNavigate('dashboard')} tone="primary">
                      Open dashboard
                    </InlineButton>
                  </div>
                </article>
                <article className="portal-shell-info-card">
                  <strong className="portal-shell-info-title">Review route posture after a security change</strong>
                  <p className="portal-shell-info-copy mt-2 text-sm">Check the Routing module to confirm the default provider posture still matches the project you intend to operate.</p>
                  <div className="mt-4">
                    <InlineButton onClick={() => onNavigate('routing')} tone="secondary">
                      Open routing
                    </InlineButton>
                  </div>
                </article>
                <article className="portal-shell-info-card">
                  <strong className="portal-shell-info-title">Separate money review from personal identity</strong>
                  <p className="portal-shell-info-copy mt-2 text-sm">When the user boundary is healthy, move into Account for cash balance, ledger evidence, and recharge posture.</p>
                  <div className="mt-4">
                    <InlineButton onClick={() => onNavigate('account')} tone="ghost">
                      Open account
                    </InlineButton>
                  </div>
                </article>
              </div>
            </Surface>
          </div>
        </TabsContent>
      </Tabs>
    </>
  );
}
