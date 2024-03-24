document.addEventListener("DOMContentLoaded", () => {
  const workerUrl = "https://func.nhrc.tito.repair-cafes.shymega.org.uk/tickets/count";
  const attendeeCountClass = "#attendee-count";

  fetch(workerUrl)
    .then(resp => {
      if (!resp.ok) {
        throw new Error("Response from Worker was not a-OK: " + resp.status);
      } else {
        return resp.json();
      };
    })
    .then(data => {
      document.getElementById(attendeeCountClass).innerHTML = "Attendees: " + data.count;
    })
    .catch(err => {
      document.getElementById(attendeeCountClass).innerHTML = ""; // Replace element with blank string.
      // Then log the error to the console.
      console.error("Error fetching attendee count: ", err);
      console.error("If you are seeing this message, please contact the site administrator.")
    });
});
