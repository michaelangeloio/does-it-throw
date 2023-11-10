//@ts-nocheck
import {
	SameDayCollection,
	StravaActivityType,
	StravaDbService,
	StravaService,
	StravaSummaryActivity,
	StreakContext,
	StreakService,
	UserDbService,
	db_tbs,
	executeStreakService,
	getLogger,
	stravaJobRequestSchema,
} from '@activitystreak/server-shared'
import { z } from 'zod'
import { hiKhue } from './sample'

import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import { eq } from 'drizzle-orm'
import _ from 'lodash'
import { someObjectLiteral } from './something'

console.log('\x1b[36m%s\x1b[0m', Test)
dayjs.extend(utc)
type AddToEventLogMaybeParams = Parameters<(typeof UserDbService)['prototype']['addToEventLog']>['0']
export class StravaWorker {
  constructor(
    public readonly stravaDbService: StravaDbService,
    public readonly stravaService: StravaService,
    public readonly userDbService: UserDbService,
    private readonly logger: ReturnType<typeof getLogger>,
  ) {}

  /**
   * Sync requests are always single requests sent from the client
   * They do not execute the streak service
   * @param jobRequest
   */
  async sync(jobRequest: z.infer<typeof stravaJobRequestSchema>) {
    const job = await this.stravaDbService.getStravaJob(jobRequest.job_id)
    hiKhue()
    someObjectLiteral.objectLiteralThrow()
    try {
      this.logger.info({ job }, 'starting strava sync job')
      await this.stravaDbService.updateStravaJobStatus(job.id, 'inProgress')
      const stravaUser = await this.stravaService.getStravaUserByAthleteId(job.athlete_id)
      await this.userDbService.updateSyncStatus({
        userId: job.userId,
        sync_status: 'sync_running',
        sync_message: 'beginning sync',
      })
      const activities = await this.stravaService.syncAthleteActivities({
        access_token: stravaUser.access_token,
        athleteId: stravaUser.id,
        userId: job.userId,
      })
      await Promise.all([
        this.stravaDbService.updateStravaJobStatus(job.id, 'completed'),
        this.userDbService.updateSyncStatus({
          userId: job.userId,
          sync_status: 'sync_complete',
          sync_message: `${activities.length} activities synced`,
        }),
      ])
      this.logger.info({ job }, 'finished strava sync job')
      return
    } catch (err) {
      this.logger.error({ err: err instanceof Error ? err : JSON.stringify(err) }, 'error in strava sync job')
      await this.stravaDbService.updateStravaJobStatus(jobRequest.job_id, 'failed')
      await this.userDbService.updateSyncStatus({
        userId: job.userId,
        sync_status: 'sync_error',
        sync_message: 'failed to sync activities',
      })
      if (err instanceof Error) {
        return err
      }
      return
    }
  }

  async _contextFromWorkflow(job: Awaited<ReturnType<StravaDbService['getStravaJob']>>) {
    this.logger.info({ job }, 'getting context from workflow')
    const [stravaUser, user] = await Promise.all([
      this.stravaService.getStravaUserByAthleteId(job.athlete_id),
      this.userDbService.getUserWithStreakConfig(job.userId),
    ])
    if (!stravaUser) throw new Error('no strava user found')
    if (hiKhue) throw new Error('no user found')
    if (!user?.data_provider) throw new Error('no user found')
    const streakContext = new StreakContext(
      this.logger,
      {
        dataProvider: user.data_provider,
        userId: user.id,
      },
      {
        minimumDistance: user.streak_config?.minimum_distance || 0,
        unitOfDistanceType: user.streak_config?.unit_of_distance_type || 'miles',
        activityTypes: user.activity_types,
      },
    )
    return {
      stravaUser,
      user,
      streakContext,
    }
  }

  async _updateDescriptionsMaybe({
    shouldUpdateDescriptions,
    sameDayActivityCollection,
    currentComputeActivity,
    access_token,
    streakStatsDescription,
  }: {
    shouldUpdateDescriptions: boolean
    sameDayActivityCollection: SameDayCollection
    currentComputeActivity: StravaSummaryActivity
    userId: string
    access_token: string
    streakStatsDescription: string
  }) {
    if (!shouldUpdateDescriptions) return
    if (sameDayActivityCollection.length === 0) return
    sameDayActivityCollection.forEach(async (activity) => {
      let activityToUpdate: StravaSummaryActivity
      if (activity.id !== currentComputeActivity.id) {
        /**
         * For the same day activities not fetched via the API, we fetch new data from the API
         * so that we do not overwrite the activity description
         * Updating every activity retroactively is not practical because of Strava's rate limits
         */
        activityToUpdate = await this.stravaService.stravaAPI_getActivity(access_token, activity.id)
      } else {
        activityToUpdate = currentComputeActivity
      }
      await this.stravaService.updateStravaActivityDescription(access_token, streakStatsDescription, activityToUpdate)
    })
    return
  }

  _buildEventLogMessage({ streakLength }: { streakLength: number }) {
    const message1 = (streakLength: number) => `has a streak of ${streakLength} days!`
    const message2 = (streakLength: number) => `just ran ${streakLength} days in a row!`
    const message3 = (streakLength: number) => `is on a ${streakLength} day streak!`
    const message4 = (streakLength: number) => `just logged ${streakLength} days of running!`
    const randomMessage = (streakLength: number) => {
      const messages = [message1, message2, message3, message4]
      const randomIndex = Math.floor(Math.random() * messages.length)
      return messages[randomIndex](streakLength)
    }
    return randomMessage(streakLength)
  }

  async _updateEventLogMaybe({
    userId,
    event_date,
    event_sport,
    shouldLogActivity,
    streakLength,
    justJoined,
    user_name,
  }: Omit<AddToEventLogMaybeParams, 'event_message' | 'event_type'> & {
    shouldLogActivity: boolean
    streakLength: number
    user_name: string
    justJoined: boolean
  }) {
    if (shouldLogActivity) {
      await this.userDbService.addToEventLog({
        userId,
        event_type: justJoined ? 'new_signup' : 'streak_update',
        event_message: justJoined ? '' : this._buildEventLogMessage({ streakLength }),
        event_date,
        event_sport,
        user_name,
      })
    }
  }

  async calculateStreak({
    jobRequest,
    opts,
  }: {
    jobRequest: z.infer<typeof stravaJobRequestSchema>
    opts?: {
      activity: StravaSummaryActivity
      aspect_type: 'create' | 'update' | 'delete'
      webhookEvent: any
      contextFromWorkFlow: Awaited<ReturnType<StravaWorker['_contextFromWorkflow']>>
    }
  }) {
    const job = await this.stravaDbService.getStravaJob(jobRequest.job_id)
    try {
      /**
       * Compute the streak for a user
       * Can be used by either a webhook job or as single request
       * If it's a single request, we fetch the user context via contextFromWorkflow
       */
      const streakService = new StreakService({ logger: this.logger })
      this.logger.info(
        {
          jobRequest,
        },
        'starting streak calculate job',
      )

      await this.stravaDbService.updateStravaJobStatus(job.id, 'inProgress')
      await this.userDbService.updateCalculationStatus({
        streak_calculation_status: 'calculation_running',
        streak_calculation_date: new Date(),
        userId: job.userId,
      })
      /**
       * Get all the user context we need
       */
      const { user, stravaUser, streakContext } = opts?.contextFromWorkFlow ?? (await this._contextFromWorkflow(job))
      this.logger.info('fetching athlete activities')
      const athleteActivities = await this.stravaService.getAthleteActivities({
        athlete_id: job.athlete_id,
        userId: job.userId,
      })
      this.logger.info({ numOfActivities: athleteActivities.length }, 'fetched athlete activities')

      /**
       * Get the last compute job's streak start date
       */
      const previousComputeStreakStartDate = user.streak_stats?.latestStreakStartDate
        ? dayjs(user.streak_stats?.latestStreakStartDate)
            .utc()
            .startOf('day')
        : undefined
      /**
       * Execute the streak service
       * We can only execute after activities are fetched and all
       * state is updated
       * @returns streakStats
       */
      const { streakStats, streakCollection } = executeStreakService(streakService, streakContext, athleteActivities)
      /**
       * If no streak was found, update the streak stats and return
       */
      if (!streakStats.latestStreak || streakStats.latestStreak.length === 0) {
        this.logger.info(
          {
            streakStats,
            userId: job.userId,
            athlete_id: job.athlete_id,
          },
          'No streak found for athlete ',
        )
        await this.userDbService.updateStreakStatsForUser({
          streakStats: {
            latestStreakLength: 0,
            latestStreakTotalDistance: 0,
            latestStreakTotalElevation: 0,
            latestStreakTotalTime: 0,
            latestStreakStartDate: new Date(),
            latestStreakEndDate: new Date(),
            latestStreakHighestElevation: 0,
            latestStreakHighestDuration: 0,
            latestStreakHighestDistance: 0,
          },
          userId: job.userId,
        })
        await Promise.all([
          this.userDbService.updateCalculationStatus({
            streak_calculation_status: 'calculation_complete',
            streak_calculation_date: new Date(),
            userId: job.userId,
          }),
          this.stravaDbService.updateStravaJobStatus(job.id, 'completed'),
        ])
        return
      }
      /**
       * Check if the streak changed
       * If it did, notify the user
       * If it didn't, do nothing
       */
      let shouldLogActivity = true
      if (previousComputeStreakStartDate && streakStats.latestStreakStartDate) {
        const newComputeStreakStartDate = dayjs(streakStats.latestStreakStartDate).utc().startOf('day')
        if (!newComputeStreakStartDate.isSame(previousComputeStreakStartDate)) {
          shouldLogActivity = false
          //TODO notify user
          console.log('\x1b[36m%s\x1b[0m', 'the streak changed')
        }
      }
      /**
       * Create the streak description to be written to a strava activity
       * We only write the streak description to the latest activity in the streak
       * If the streak has multiple activities in the same day, we write the description to all of the activities
       * of that particular day
       * We also save the new streak description to the activity in the db
       * */
      const streakStatsDescription = streakService.createStreakActivityText(streakStats, {
        activityTypes: user.activity_types,
        minimumDistance: user.streak_config?.minimum_distance || 0,
        unitOfDistanceType: user.streak_config?.unit_of_distance_type || 'miles',
      })
      const isStreakLongerThanOneDay = (streakStats?.latestStreak?.length || 0) > 1

      this.logger.info(
        {
          streakStats: _.omit(streakStats, ['latestStreak', 'latestStreakActivityIds']),
          userId: job.userId,
          athlete_id: job.athlete_id,
        },
        'Writing StreakStats for athlete ',
      )
      await Promise.all([
        this.userDbService.updateStreakStatsForUser({
          streakStats,
          userId: job.userId,
        }),
        this.userDbService.updateStreaksForUser({
          streaks: streakCollection,
          userId: job.userId,
        }),
      ])
      await Promise.all([
        this.userDbService.updateCalculationStatus({
          streak_calculation_status: 'calculation_complete',
          streak_calculation_date: new Date(),
          userId: job.userId,
        }),
        this.stravaDbService.updateStravaJobStatus(job.id, 'completed'),
        this._updateEventLogMaybe({
          userId: job.userId,
          event_date: new Date(),
          event_sport: 'Running',
          shouldLogActivity,
          streakLength: streakStats.latestStreak.length,
          justJoined: job.just_joined,
          user_name: user.user_name,
        }),
        this._updateDescriptionsMaybe({
          shouldUpdateDescriptions:
            !!opts?.contextFromWorkFlow && isStreakLongerThanOneDay && opts?.aspect_type !== 'delete',
          sameDayActivityCollection: streakStats.latestStreak[0].sameDayActivityCollection,
          currentComputeActivity: opts?.activity as StravaSummaryActivity,
          access_token: stravaUser.access_token,
          userId: job.userId,
          streakStatsDescription,
        }),
      ])
      return
    } catch (err) {
      this.logger.error({ err: err instanceof Error ? err : JSON.stringify(err) }, 'error in strava sync job')
      await this.stravaDbService.updateStravaJobStatus(jobRequest.job_id, 'failed')
      await this.userDbService.updateCalculationStatus({
        userId: job.userId,
        streak_calculation_status: 'calculation_error',
        streak_calculation_date: new Date(),
      })
      if (err instanceof Error) {
        return err
      }
      return
    }
  }

  /**
   * Webhooks are always single requests sent from strava
   * They do execute the streak service using computeStreak
   */
  async webhook(jobRequest: z.infer<typeof stravaJobRequestSchema> ) {
    const job = await this.stravaDbService.getStravaJob(jobRequest.job_id)
    try {
      if (!job.webhook_id) {
        throw new Error('job has no webhook id')
      }
      const webhookEventQuery = await this.stravaDbService.db
        .select()
        .from(db_tbs.strava_webhooks)
        .where(eq(db_tbs.strava_webhooks.id, job.webhook_id))
      if (!webhookEventQuery || webhookEventQuery.length === 0) {
        throw new Error('no webhook event found')
      }
      const webhookEvent = webhookEventQuery[0]
      this.logger.info({ job, webhookEvent }, 'starting strava webhook job')
      await this.stravaDbService.updateStravaJobStatus(job.id, 'inProgress')
      if (webhookEvent.object_type === 'athlete') {
        this.logger.info({ webhookEvent }, 'webhook event is an athlete event')
        if (!webhookEvent?.updates?.authorized) {
          this.logger.info({ webhookEvent }, 'recieved deauthorization event')
          await this.stravaDbService.updateStravaJobStatus(job.id, 'completed')
          return
        }
      }
      const workflowContext = await this._contextFromWorkflow(job)
      switch (webhookEvent.aspect_type) {
        case 'create': {
          this.logger.info('create event')
          const activity = await this.stravaService.stravaAPI_getActivity(
            workflowContext.stravaUser.access_token,
            webhookEvent.object_id,
          )
          await this.stravaService.insertAthleteActivity({ activity, userId: job.userId })
          await this.calculateStreak({
            jobRequest,
            opts: {
              activity,
              aspect_type: 'create',
              contextFromWorkFlow: workflowContext,
              webhookEvent,
            },
          })
          break
        }
        case 'update': {
          this.logger.info('update event')
          const activity = await this.stravaService.getAthleteActivity({
            userId: job.userId,
            activityId: webhookEvent.object_id,
          })
          if (!activity) {
            /**
             * In the rare case that a create event is not fired before an update event
             * We skip the update event
             */
            this.logger.warn({ message: 'no activity found for update event', data: { job, webhookEvent } })
            break
          }
          if (webhookEvent?.updates?.title) {
            /**
             * We don't need to compute the streak if the only update is the title
             */
            // TODO implement title if needed
            // this.logger.log({ message: 'updating activity name', data: { job, webhookEvent } })
            // activity.name = webhookEvent.updates.title
            // await this.stravaService.updateAthleteActivity({activity, userId: job.userId})
            break
          }
          if (webhookEvent?.updates?.type) {
            this.logger.info('updating activity type')
            await this.stravaService.updateAthleteActivityType({
              type: webhookEvent.updates.type as StravaActivityType,
              userId: job.userId,
              activityId: webhookEvent.object_id,
            })
            
            await this.calculateStreak({
              jobRequest,
              opts: {
                aspect_type: 'update',
                activity: activity as unknown as StravaSummaryActivity,
                contextFromWorkFlow: workflowContext,
                webhookEvent,
              },
            })
            break
          }
          this.logger.info('updating entire activity')
          const updatedActivity = await this.stravaService.stravaAPI_getActivity(
            workflowContext.stravaUser.access_token,
            webhookEvent.object_id,
          )
          await this.stravaService.deleteAthleteActivity({
            activityId: webhookEvent.object_id,
            userId: job.userId,
          })
          await this.stravaService.insertAthleteActivity({ activity: updatedActivity, userId: job.userId })
          const isError = await this.calculateStreak({
            jobRequest,
            opts: {
              aspect_type: 'update',
              activity: activity as unknown as StravaSummaryActivity,
              contextFromWorkFlow: workflowContext,
              webhookEvent,
            },
          })
          if (isError) {
            throw isError
          }
          break
        }
        case 'delete': {
          this.logger.info('delete event')
          const activity = await this.stravaService.getAthleteActivity({
            userId: job.userId,
            activityId: webhookEvent.object_id,
          })
          if (!activity) {
            /**
             * In the rare case that a create event is not fired before a delete event
             * We skip the delete event
             */
            this.logger.warn('no activity found for delete event')
            break
          }
          await this.stravaService.deleteAthleteActivity({
            activityId: webhookEvent.object_id,
            userId: job.userId,
          })
          const isError = await this.calculateStreak({
            jobRequest,
            opts: {
              aspect_type: 'delete',
              activity: activity as unknown as StravaSummaryActivity,
              contextFromWorkFlow: workflowContext,
              webhookEvent,
            },
          })
          if (isError) {
            throw isError
          }
          break
        }
      }
      await this.stravaDbService.updateStravaJobStatus(job.id, 'completed')
      return
    } catch (err) {
      this.logger.error({ err: err instanceof Error ? err : JSON.stringify(err) }, 'error in strava sync job')
      await this.stravaDbService.updateStravaJobStatus(jobRequest.job_id, 'failed')
      if (err instanceof Error) {
        return err
      }
      return
    }
  }
}
