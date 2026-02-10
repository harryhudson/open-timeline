//------------------------------------------------------------------------------
// Defines the <open-timeline-canvas> element
//------------------------------------------------------------------------------

export default class OpenTimelineCanvasElement extends HTMLElement {

	//--------------------------------------------------------------------------
	// We want to be notified when the "size" attribute changes
	//
	// `attributeChangedCallback()` will be used to make use of the notifications
	//
	// Not sure we really need or want this
	//--------------------------------------------------------------------------
	static observedAttributes = ["size"]

	constructor() {
		super();

		//----------------------------------------------------------------------
		// Add the <div> container
		//----------------------------------------------------------------------
		let timeline_container = document.createElement("div")
		timeline_container.setAttribute("canvas-timeline-container", "")
		this.appendChild(timeline_container)

		//----------------------------------------------------------------------
		// Add the visible canvas
		//----------------------------------------------------------------------
		let visible_canvas = document.createElement("canvas")
		visible_canvas.setAttribute("visible", "")
		timeline_container.appendChild(visible_canvas)

		//----------------------------------------------------------------------
		// Add the invisible canvas
		//----------------------------------------------------------------------
		let invisible_canvas = document.createElement("canvas")
		invisible_canvas.setAttribute("invisible", "")
		timeline_container.appendChild(invisible_canvas)

		//----------------------------------------------------------------------
		// Add styling
		//
		// Styling the <open-timeline-canvas> is awkward
		//
		// Have the div container height in here which isn't nice
		//----------------------------------------------------------------------
		let style = document.createElement('style');
		style.setAttribute("timeline-style", "");
		style.innerHTML = `
			open-timeline-canvas {
				// border: solid black 1px;
			}
			open-timeline-canvas div[canvas-timeline-container] {
				display: grid;
				// height: 500px;
				// border: solid black 1px;
			}
			open-timeline-canvas canvas[visible], canvas[invisible] {
				// grid-row: 1;
				grid-column: 1;
				// border: solid black 1px;
			}
			open-timeline-canvas canvas[invisible] {
				// visibility: hidden;
			}
		`
		document.querySelector('body').appendChild(style);
	}

	attributeChangedCallback(name, oldValue, newValue) {
		console.log(`Attribute ${name} changed: ${oldValue} -> ${newValue}`)
	}
}

customElements.define("open-timeline-canvas", OpenTimelineCanvasElement);
