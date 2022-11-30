export const makeTextObject = (string) => {
	const isInBrackets = (str: string): boolean => str.startsWith('[') && str.endsWith(']')

	let finalArray = []
	string = string.match(/(([^[\]]+)|(\[([^\]]+)\]))/g).map((val) => val === "[]" ? null : val)
	for (let i = 0; i < string.length; i++) {
		finalArray.push({
			text: isInBrackets(string[i]) ? string[i].substr(1, string[i].length - 2) : string[i],
			mark: isInBrackets(string[i]),
		})
	}
	return finalArray
}
