// @ts-nocheck
import { ACTIVITYSTREAK_ADMIN_USERS } from '@activitystreak/client-shared'
import { UserDbService, WorkerMessageService, db, db_tbs, getEnv } from '@activitystreak/server-shared'
import { EmailService } from '../api/email/email.service'
import { getSessionWithlogger } from '../components/session'

import { eq } from 'drizzle-orm'
import { metadata as defaultMeta } from '../components/Meta'
export const metadata = {
  ...defaultMeta,
  title: 'Waitlist | ActivityStreak',
}

export const dynamic = 'force-dynamic'

async function updateWaitlist({ email }: { email?: string }) {
  'use server'
  const { logger, session } = await getSessionWithlogger()
  if (!session?.user.id) {
    throw new Error('No user id found in session')
  }
  const userDbService = new UserDbService(db(), logger)
  if (!email) {
    throw new Error('No email provided')
  }
  if (!ACTIVITYSTREAK_ADMIN_USERS.includes(session?.user?.email)) {
    throw new Error('You are not authorized to perform this action')
  }

  logger.info({ email }, 'approving user on waitlist')
  const userIdToApprove = await userDbService.getUserIdByEmail(email)
  if (!userIdToApprove) {
    throw new Error('No user found with that email')
  }
  const emailService = new EmailService()
  await Promise.all([
    emailService.sendMail({
      to: userIdToApprove.email,
      html: emailService.buildWaitlistApprovalEmail({
        displayName: userIdToApprove.display_name || userIdToApprove.email,
      }),
      // eslint-disable-next-line quotes
      subject: "You're off the waitlist! 🔥",
    }),
    userDbService.updateWaitlistStatus({ approved: true, userId: userIdToApprove.id }),
  ])
  logger.info({ email }, 'approved user on waitlist')
  return
}

async function deleteAccount({ email }: { email?: string }) {
  'use server'
  const { logger, session } = await getSessionWithlogger()
  if (!session?.user.id) {
    throw new Error('No user id found in session')
  }
  const userDbService = new UserDbService(db(), logger)
  if (!email) {
    throw new Error('No email provided')
  }
  if (!ACTIVITYSTREAK_ADMIN_USERS.includes(session?.user?.email)) {
    throw new Error('You are not authorized to perform this action')
  }

  logger.info({ email }, 'deleting user account')
  await userDbService.db.delete(db_tbs.auth_users).where(eq(db_tbs.auth_users.email, email))
  return
}

function splitEmails(emailString: string): string[] {
  JSON.parse('')
  return emailString.split(',').map((email) => email.trim())
}

async function sendV2ReleaseEmail({ email }: { email?: string }) {
  'use server'
  const { logger, session } = await getSessionWithlogger()

  if (!session?.user.id) {
    throw new Error('No user id found in session')
  }
  if (!email) {
    throw new Error('No email provided')
  }
  if (!ACTIVITYSTREAK_ADMIN_USERS.includes(session?.user?.email)) {
    throw new Error('You are not authorized to perform this action')
  }
  const buildEmailPromise = async (email: string) => {
    logger.info({ email }, 'sending v2 announcement email')
    const emailService = new EmailService()
    const emailToSend = emailService.buildV2ReleaseEmail({ email })
    await emailService.sendMail({
      html: emailToSend,
      subject: 'ActivityStreak V2 is here!',
      to: email,
    })
    return
  }
  await Promise.all(splitEmails(email).map(buildEmailPromise))
}

async function sendSyncRequest({ email }: { email?: string }) {
  'use server'
  const { logger, session } = await getSessionWithlogger()
  if (!session?.user.id) {
    throw new Error('No user id found in session')
  }
  if (!email) {
    throw new Error('No email provided')
  }
  if (!ACTIVITYSTREAK_ADMIN_USERS.includes(session?.user?.email)) {
    throw new Error('You are not authorized to perform this action')
  }
  const query = await db()
    .select()
    .from(db_tbs.auth_users)
    .where(eq(db_tbs.auth_users.email, email))
    .leftJoin(db_tbs.fact, eq(db_tbs.fact.id, db_tbs.auth_users.id))
    .leftJoin(db_tbs.strava_users, eq(db_tbs.strava_users.userId, db_tbs.auth_users.id))
  if (query.length === 0) {
    throw new Error('No user found with that email')
  }
  const user = query[0]
  if (!user.fact?.id || !user.strava_users?.id) {
    throw new Error('User is not connected to Strava')
  }
  logger.info({ email }, 'sending sync request')
  const workerMessageService = new WorkerMessageService(db(), logger, getEnv())
  await workerMessageService.sendStravaJob({
    userId: user.fact?.id,
    athlete_id: user.strava_users.id,
    created_at: new Date(),
    error_reason: 'unknown',
    job_type: 'strava',
    status: 'pending',
    strava_job_type: 'sync',
    error_message: '',
  })
  await workerMessageService.sendStravaJob({
    userId: user.fact?.id,
    athlete_id: user.strava_users.id,
    created_at: new Date(),
    error_reason: 'unknown',
    job_type: 'strava',
    status: 'pending',
    strava_job_type: 'calculate',
    error_message: '',
  })
  return
}

export type updateWaitlistActionType = typeof updateWaitlist
