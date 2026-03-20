import init, * as OpenTimelineGames from "./../../pkg/open_timeline_games.js";
await init();

// Log the object
console.log(OpenTimelineGames);

// Entities with a start year after 1600
export async function get_entities() {
	const entities = await all_entities();
	console.log("First 4 entities: ", entities.slice(0, 4))
	const filtered = entities.filter(entity => entity.start.year > 1600);
	return filtered.slice(0, 4)
}

// Read all entities from entities.json
export async function all_entities() {
	const res = await fetch('../other/entities.json');
	return await res.json();
}

// Print game stats
export function print_stats(stats) {
	console.log(`Stats: Round = ${stats.round}`)
	console.log(`Stats: Correct = ${stats.correct_round_count}`)
	console.log(`Stats: Incorrect = ${stats.incorrect_round_count}`)
}

// Is the answer correct
export function answer_is_correct(answer) {
	return answer == OpenTimelineGames.Answer.Correct
}
