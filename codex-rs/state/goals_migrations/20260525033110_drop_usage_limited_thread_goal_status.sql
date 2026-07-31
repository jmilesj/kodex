UPDATE thread_goals
SET status = 'active'
WHERE status = 'usage_limited';

CREATE TRIGGER thread_goals_reject_usage_limited_insert
BEFORE INSERT ON thread_goals
WHEN NEW.status = 'usage_limited'
BEGIN
    SELECT RAISE(ABORT, 'usage_limited thread goal status is no longer supported');
END;

CREATE TRIGGER thread_goals_reject_usage_limited_update
BEFORE UPDATE OF status ON thread_goals
WHEN NEW.status = 'usage_limited'
BEGIN
    SELECT RAISE(ABORT, 'usage_limited thread goal status is no longer supported');
END;
