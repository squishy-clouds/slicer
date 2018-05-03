//! A simple, efficient utility for slicing string slices into smaller string 
//! slices. Useful for parsing anything represented by strings, such as
//! programming languages or data formats.
//! 
//! ## Examples
//! 
//! Basic usage:
//! 
//! ```
//! # use slicer::AsSlicer;
//! let path = "images/cat.jpeg";
//! let mut slicer = path.as_slicer();
//! 
//! let directory = slicer.slice_until("/");
//! slicer.skip_over("/");
//! let filename = slicer.slice_until(".");
//! slicer.skip_over(".");
//! let extension = slicer.slice_to_end();
//! 
//! assert_eq!(Some("images"), directory);
//! assert_eq!(Some("cat"), filename);
//! assert_eq!(Some("jpeg"), extension);
//! ```

/// Describes a type that can be cheaply converted into a [`StrSlicer`].
///
/// [`StrSlicer`]: struct.StrSlicer.html
pub trait AsSlicer<'str> {
    /// Converts the type to a [`StrSlicer`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This string will turn into a slicer".as_slicer();
    /// ```
    ///
    /// [`StrSlicer`]: struct.StrSlicer.html
    fn as_slicer(&self) -> StrSlicer<'str>;
    /// Converts the type to a slicer with the given [`Tracker`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = "This string will turn into a slicer using the given tracker".as_slicer_with_tracker(LineTracker::new());
    /// ```
    ///
    /// [`Tracker`]: trait.Tracker.html
    fn as_slicer_with_tracker<T: Tracker>(&'str self, tracker: T) -> StrSlicer<'str, T>;
}
impl<'str> AsSlicer<'str> for &'str str {
    fn as_slicer(&self) -> StrSlicer<'str> {
        StrSlicer::new(self)
    }
    fn as_slicer_with_tracker<T: Tracker>(&self, tracker: T) -> StrSlicer<'str, T> {
        StrSlicer::with_tracker(self, tracker)
    }
}

/// Describes a type that tracks information as the [`StrSlicer`] goes through the string.
///
/// [`StrSlicer`]: struct.StrSlicer.html
pub trait Tracker {
    /// Type of the position returned from [`StrSlicer::tracker_pos`].
    ///
    /// [`StrSlicer::tracker_pos`]: struct.StrSlicer.html#method.tracker_pos
    type Pos;
    /// Returns the position information tracked by this tracker. Called by [`StrSlicer::tracker_pos`].
    ///
    /// [`StrSlicer::tracker_pos`]: struct.StrSlicer.html#method.tracker_pos
    fn pos(&self) -> Self::Pos;
    /// Updates the position information tracked by this tracker. Called internally when the [`StrSlicer`] changes its position, such as when [`jump_to`] or [`jump_to_unchecked`] are called.
    ///
    /// [`StrSlicer`]: struct.StrSlicer.html
    /// [`jump_to`]: struct.StrSlicer.html#method.jump_to
    /// [`jump_to_unchecked`]: struct.StrSlicer.html#method.jump_to_unchecked
    fn update(&mut self, string: &str, old_byte_pos: usize, new_byte_pos: usize);
}
/// Allows the `()` type to be used as a null tracker, that doesn't do anything.
impl Tracker for () {
    type Pos = ();
    fn pos(&self) -> Self::Pos {
        ()
    }
    fn update(&mut self, _string: &str, _old_byte_pos: usize, _new_byte_pos: usize) {}
}

/// Describes a type that can be used as an input to many of [`StrSlicer`]'s methods.
///
/// [`StrSlicer`]: struct.StrSlicer.html
pub trait Pattern {
    /// Checks whether the pattern is found in the given [`StrSlicer`] at its current postion.
    ///
    /// See [`StrSlicer::is_next`] for more details.
    ///
    /// [`StrSlicer`]: struct.StrSlicer.html
    /// [`StrSlicer::is_next`]: struct.StrSlicer.html#method.is_next
    fn is_next<'str, T: Tracker>(&mut self, slicer: &StrSlicer<'str, T>) -> bool;
    /// Steps the given [`StrSlicer`] ahead until this pattern is next, or until the end of string is hit.
    ///
    /// See [`StrSlicer::skip_until`] and [`StrSlicer::slice_until`] for more details.
    ///
    /// [`StrSlicer`]: struct.StrSlicer.html
    /// [`StrSlicer::skip_until`]: struct.StrSlicer.html#method.skip_until
    /// [`StrSlicer::slice_until`]: struct.StrSlicer.html#method.slice_until
    fn skip_until<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>);
    /// Steps the given [`StrSlicer`] over this pattern. Doesn't check if the pattern is actually next.
    ///
    /// See [`StrSlicer::skip_over`] for more details.
    ///
    /// [`StrSlicer`]: struct.StrSlicer.html
    /// [`StrSlicer::skip_over`]: struct.StrSlicer.html#method.skip_over
    unsafe fn skip_over_unchecked<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>);
}
impl<'a> Pattern for &'a str {
    fn is_next<'str, T: Tracker>(&mut self, slicer: &StrSlicer<'str, T>) -> bool {
        /*let start_pos = slicer.byte_pos();
        let end_pos = start_pos + self.len();
        if end_pos >= slicer.end_byte_pos() {
            false
        } else {
            *self == &slicer.string[start_pos..end_pos]
        }*/
        match slicer.cut_off() {
            None => false,
            Some(cut_off) => cut_off.starts_with(*self)
        }
    }
    fn skip_until<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        let cut_off = match slicer.cut_off() {
            None => return, //return early, since the slicer is finished so there's nothing we can do
            Some(cut_off) => cut_off
        };
        match cut_off.find(*self) {
            //if this pattern was not found in the string, simulate skipping until the end of the string
            None => slicer.skip_to_end(),
            //if the pattern was found, jump to it
            Some(offset) => {
                let byte_pos = slicer.byte_pos;
                unsafe {
                    slicer.jump_to_unchecked(byte_pos + offset);
                }
            }
        }
    }
    unsafe fn skip_over_unchecked<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        let byte_pos = slicer.byte_pos;
        slicer.jump_to_unchecked(byte_pos + self.len());
    }
}
impl Pattern for char {
    fn is_next<'str, T: Tracker>(&mut self, slicer: &StrSlicer<'str, T>) -> bool {
        match slicer.as_str().chars().next() {
            Some(char) => *self == char,
            None => false
        }
    }
    fn skip_until<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        let cut_off = match slicer.cut_off() {
            None => return, //return early, since the slicer is finished so there's nothing we can do
            Some(cut_off) => cut_off
        };
        match cut_off.find(*self) {
            //if this pattern was not found in the string, simulate skipping until the end of the string
            None => slicer.skip_to_end(),
            //if the pattern was found, jump to it
            Some(offset) => {
                let byte_pos = slicer.byte_pos;
                unsafe {
                    slicer.jump_to_unchecked(byte_pos + offset);
                }
            }
        }
    }
    unsafe fn skip_over_unchecked<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        let byte_pos = slicer.byte_pos;
        slicer.jump_to_unchecked(byte_pos + self.len_utf8());
    }
}
impl<F: FnMut(char) -> bool> Pattern for F {
    fn is_next<'str, T: Tracker>(&mut self, slicer: &StrSlicer<'str, T>) -> bool {
        match slicer.as_str().chars().next() {
            Some(char) => self(char),
            None => false
        }
    }
    fn skip_until<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        let cut_off = match slicer.cut_off() {
            None => return, //return early, since the slicer is finished so there's nothing we can do
            Some(cut_off) => cut_off
        };
        match cut_off.find(self) {
            //if this pattern was not found in the string, simulate skipping until the end of the string
            None => slicer.skip_to_end(),
            //if the pattern was found, jump to it
            Some(offset) => {
                let byte_pos = slicer.byte_pos;
                unsafe {
                    slicer.jump_to_unchecked(byte_pos + offset)
                }
            }
        }
    }
    unsafe fn skip_over_unchecked<'str, T: Tracker>(&mut self, slicer: &mut StrSlicer<'str, T>) {
        slicer.advance_char();
    }
}

/// A string slicer.
///
/// Walks over a string slice, slicing it into smaller string slices.
///
/// Slicing methods (those titled `slice_…`) return an `None` once the slicer
/// has walked to the end, which makes it easy to avoid infinite loops.
#[derive(Debug, Clone, Copy)]
pub struct StrSlicer<'str, T: Tracker = ()> {
    string: &'str str,
    byte_pos: usize,
    tracker: T
}
impl<'str> StrSlicer<'str, ()> {
    /// Creates a `StrSlicer` from the given string slice.
    ///
    /// You should prefer to use [`AsSlicer::as_slicer`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::StrSlicer;
    /// let mut slicer = StrSlicer::new("This string is being turned into a string slicer.");
    /// ```
    ///
    /// [`AsSlicer::as_slicer`]: trait.AsSlicer.html#tymethod.as_slicer.html
    pub fn new(string: &'str str) -> Self {
        Self {
            string,
            byte_pos: 0,
            tracker: ()
        }
    }
}
impl<'str, T: Tracker> StrSlicer<'str, T> {
    /// Creates a `StrSlicer` from the given string slice and [`Tracker`].
    ///
    /// You should prefer to use [`AsSlicer::as_slicer_with_tracker`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::StrSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = StrSlicer::with_tracker("This string is being turned into a string slicer.", LineTracker::new());
    /// ```
    ///
    /// [`AsSlicer::as_slicer_with_tracker`]: trait.AsSlicer.html#tymethod.as_slicer_with_tracker.html
    /// [`Tracker`]: trait.Tracker.html
    pub fn with_tracker(string: &'str str, tracker: T) -> Self {
        Self {
            string: string,
            byte_pos: 0,
            tracker
        }
    }
    
    fn next_char_boundary(&self) -> Option<usize> {
        let mut next_byte_pos = self.byte_pos + 1;
        loop {
            if next_byte_pos >= self.end_byte_pos() {
                return None;
            }
            
            if self.string.is_char_boundary(next_byte_pos) {
                return Some(next_byte_pos);
            } else {
                next_byte_pos += 1;
                continue;
            }
        }
    }
    fn advance_char(&mut self) {
        let byte_pos = self.next_char_boundary().unwrap_or(self.end_byte_pos());
        unsafe {
            self.jump_to_unchecked(byte_pos);
        }
    }
    #[inline]
    fn end_byte_pos(&self) -> usize {
        self.string.len()
    }
    
    /// Returns a reference to the string slice that this slicer is operating on.
    ///
    /// The returned strign slice has the same lifetime as the slicer itself.
    ///
    /// `StrSlicer` also implments the standard trait [`AsRef<str>`](https://doc.rust-lang.org/nightly/std/convert/trait.AsRef.html),
    /// which does the same thing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let string = "...";
    /// let slicer = string.as_slicer();
    ///
    /// assert_eq!(string, slicer.as_str());
    /// ```
    #[inline]
    pub fn as_str(&self) -> &'str str {
        self.string
    }
    /// Cuts off the end of the string slice at the current position and returns that slice,
    /// without also jumping ahead to the end, as [`slice_to_end`] does.
    ///
    /// The returned string slice has the same lifetime as the slicer itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "beepboop".as_slicer();
    /// slicer.jump_to(4);
    /// assert_eq!(slicer.cut_off(), Some("boop"));
    /// ```
    ///
    /// [`slice_to_end`]: struct.StrSlicer.html#method.slice_to_end
    pub fn cut_off(&self) -> Option<&'str str> {
        if self.is_at_end() {
            None
        } else {
            let start_pos = self.byte_pos;
            let end_pos = self.end_byte_pos();
            Some(&self.string[start_pos..end_pos])
        }
    }
    
    /// Gets the slicer's current position in the string as a byte index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "Violets are the best flower.".as_slicer();
    /// slicer.jump_to(5);
    /// assert_eq!(slicer.byte_pos(), 5);
    /// ```
    #[inline]
    pub fn byte_pos(&self) -> usize {
        self.byte_pos
    }
    /// Jumps the slicer to the given byte index
    ///
    /// # Panics
    ///
    /// Panics if `byte_pos` is not on a UTF-8 code point boundary, or if it is
    /// beyond the end of the string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "Violets are the best flower.".as_slicer();
    /// slicer.jump_to(5);
    /// assert_eq!(slicer.byte_pos(), 5);
    /// ```
    ///
    /// Jumping to the middle of a UTF-8 codepoint panics. This example panics:
    ///
    /// ```should_panic
    /// # use slicer::AsSlicer;
    /// let mut slicer = "🌺 is a hibiscus.".as_slicer();
    /// slicer.jump_to(2); //the hibiscus emoji is 4 bytes long, so jumping to the middle of it panics.
    /// ```
    pub fn jump_to(&mut self, byte_pos: usize) {
        if byte_pos > self.end_byte_pos() {
            jump_oob_fail(self.string, byte_pos);
        }
        if self.string.is_char_boundary(byte_pos) {
            unsafe {
                self.jump_to_unchecked(byte_pos);
            }
        } else {
            jump_char_boundary_fail(self.string, byte_pos)
        }
    }
    /// Equivalent to [`jump_to`], except without any bounds checking.
    ///
    /// You should almost always prefer to use [`jump_to`].
    ///
    /// # Safety
    ///
    /// This function will never panic, although if `byte_pos` is not on a UTF-8
    /// code point boundary, the slicer will be left in an illegal state and may panic on later method calls.
    ///
    /// Jumping beyond the last code point of the string slice, however, will not leave
    /// the slicer in an illegal state, it will act the same as if [`skip_to_end`] was called.
    ///
    /// [`jump_to`]: struct.StrSlicer.html#method.jump_to
    /// [`skip_to_end`]: struct.StrSlicer.html#method.skip_to_end
    pub unsafe fn jump_to_unchecked(&mut self, byte_pos: usize) {
        let string = self.as_str();
        self.tracker.update(string, self.byte_pos, byte_pos);
        self.byte_pos = byte_pos;
    }
    
    /// Returns a reference to this slicer's tracker.
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = "This string is being turned into a string slicer.".as_slicer_with_tracker(LineTracker::new());
    /// let tracker = slicer.tracker();
    /// ```
    pub fn tracker(&self) -> &T {
        &self.tracker
    }
    /// Returns a mutable reference to this slicer's tracker.
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = "This string is being turned into a string slicer.".as_slicer_with_tracker(LineTracker::new());
    /// let tracker = slicer.tracker_mut();
    /// ```
    pub fn tracker_mut(&mut self) -> &mut T {
        &mut self.tracker
    }
    /// Gets the position value that this slicer's [`Tracker`] is tracking.
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = "Line 1\nLine 2\nLine 3".as_slicer_with_tracker(LineTracker::new());
    /// slicer.skip_to_end();
    /// assert_eq!(slicer.tracker_pos(), 2);
    /// ```
    ///
    /// [`Tracker`]: trait.Tracker.html
    #[inline]
    pub fn tracker_pos(&self) -> T::Pos {
        self.tracker.pos()
    }
    
    //pub fn skip_num_bytes(&mut self, num: usize);
    //pub fn slice_num_bytes(&mut self, num: usize) -> Option<&'str str>;
    
    /// Skips over `num` chars in this slicer's string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "🌷 Tulip!".as_slicer();
    /// slicer.skip_num_chars(1);
    /// assert_eq!(slicer.cut_off(), Some(" Tulip!"));
    /// ```
    pub fn skip_num_chars(&mut self, num: usize) {
        for _ in 0..num {
            self.advance_char();
            if self.is_at_end() {
                break;
            }
        }
    }
    /// Skips over `num` chars in this slicer's string, and returns the area skipped over as a string slice.
    ///
    /// Returns `None` if this slicer is past the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "🌷 For a period in 1636-1637, tulips were considered extremely valuable.".as_slicer();
    /// assert_eq!(slicer.slice_num_chars(1), Some("🌷"));
    /// ```
    pub fn slice_num_chars(&mut self, num: usize) -> Option<&'str str> {
        let start_pos = self.byte_pos;
        if start_pos >= self.end_byte_pos() {
            None
        } else {
            self.skip_num_chars(num);
            let end_pos = self.byte_pos;
            Some(&self.string[start_pos..end_pos])
        }
    }
    
    /// Checks whether or not the given [`Pattern`] is next.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "123456".as_slicer();
    /// assert_eq!(slicer.skip_over("123"), true);
    /// assert_eq!(slicer.is_next("456"), true);
    /// ```
    pub fn is_next<P: Pattern>(&self, mut pattern: P) -> bool {
        pattern.is_next(self)
    }
    
    /// Checks whether or not the given [`Pattern`] is next, if its next, it skips over
    /// the pattern and returns true, if its not it does nothing and returns false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "123456".as_slicer();
    /// if slicer.skip_over("123") {
    ///     assert_eq!(slicer.is_next("456"), true);
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    ///
    /// [`Pattern`]: trait.Pattern.html
    pub fn skip_over<P: Pattern>(&mut self, mut pattern: P) -> bool {
        if pattern.is_next(self) {
            unsafe {
                pattern.skip_over_unchecked(self);
            }
            true
        } else {
            false
        }
    }
    /// Skips over the given [`Pattern`] without checking to see if its actually next.
    ///
    /// You should almost always prefer to use [`skip_over`].
    ///
    /// [`skip_over`]: struct.StrSlicer.html#method.skip_over
    /// [`Pattern`]: trait.Pattern.html
    pub unsafe fn skip_over_unchecked<P: Pattern>(&mut self, mut pattern: P) {
        pattern.skip_over_unchecked(self)
    }
    
    /// Skips forward until the given [`Pattern`] is next.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// slicer.skip_until("sentence");
    /// assert_eq!(slicer.is_next("sentence"), true);
    /// ```
    ///
    /// [`Pattern`]: trait.Pattern.html
    pub fn skip_until<P: Pattern>(&mut self, mut pattern: P) {
        pattern.skip_until(self);
    }
    /// Skips forward until the given [`Pattern`] is next, and returns the area skipped over as a string slice.
    ///
    /// Returns `None` if this slicer is past the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// assert_eq!(slicer.slice_until("sentence"), Some("This is a "));
    /// ```
    ///
    /// [`Pattern`]: trait.Pattern.html
    pub fn slice_until<P: Pattern>(&mut self, pattern: P) -> Option<&'str str> {
        let start_pos = self.byte_pos;
        if start_pos >= self.end_byte_pos() {
            None
        } else {
            self.skip_until(pattern);
            let end_pos = self.byte_pos;
            Some(&self.string[start_pos..end_pos])
        }
    }
    
    /// Skips forward until the given [`Pattern`] is next, then skips over the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// slicer.skip_until_after("sentence");
    /// assert_eq!(slicer.is_next("."), true);
    /// ```
    ///
    /// [`Pattern`]: trait.Pattern.html
    pub fn skip_until_after<P: Pattern>(&mut self, mut pattern: P) {
        pattern.skip_until(self);
        if !self.is_at_end() {
            //`skip_until` skips through the string until the pattern is found, so we're safe to
            //assume the pattern is next and we don't need to use the checked version of `skip_over`
            unsafe {
                pattern.skip_over_unchecked(self);
            }
        }
    }
    /// Skips forward until the given [`Pattern`] is next, then skips over the pattern and returns the area skipped over as a string slice.
    ///
    /// Returns `None` if this slicer is past the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// assert_eq!(slicer.slice_until("sentence"), Some("This is a "));
    /// ```
    ///
    /// [`Pattern`]: trait.Pattern.html
    pub fn slice_until_after<P: Pattern>(&mut self, pattern: P) -> Option<&'str str> {
        let start_pos = self.byte_pos;
        if start_pos >= self.end_byte_pos() {
            None
        } else {
            self.skip_until_after(pattern);
            let end_pos = self.byte_pos;
            Some(&self.string[start_pos..end_pos])
        }
    }
    
    /// Skips forward until a non-whitespace character is next.
    ///
    /// If a non-whitespace character is already next, nothing is done.
    ///
    /// Equivalent to `skip_until(|char: char| !char.is_whitespace())`
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a flower 🌹.".as_slicer();
    /// slicer.skip_over("This");
    /// slicer.skip_whitespace();
    /// assert_eq!(slicer.is_next("is"), true);
    /// ```
    pub fn skip_whitespace(&mut self) {
        self.skip_until(|char: char| !char.is_whitespace());
    }
    /// Skips forward until a non-whitespace character is next, and returns the area skipped over as a string slice.
    ///
    /// If a non-whitespace character is already next, nothing is done.
    ///
    /// Returns `None` if this slicer is past the end of the string.
    ///
    /// Equivalent to `slice_until(|char: char| !char.is_whitespace())`
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a flower 🌹.".as_slicer();
    /// slicer.skip_over("This");
    /// assert_eq!(slicer.slice_whitespace(), Some(" "));
    /// ```
    pub fn slice_whitespace(&mut self) -> Option<&'str str> {
        self.slice_until(|char: char| !char.is_whitespace())
    }
    
    /// Skips forward until a whitespace character is next.
    ///
    /// If a whitespace character is already next, nothing is done.
    ///
    /// Opposite of [`skip_whitespace`].
    ///
    /// Equivalent to `skip_until(|char: char| char.is_whitespace())`
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// slicer.skip_non_whitespace();
    /// assert_eq!(slicer.is_next(" is"), true);
    /// ```
    ///
    /// [`skip_whitespace`]: struct.StrSlicer.html#method.skip_whitespace
    pub fn skip_non_whitespace(&mut self) {
        self.skip_until(|char: char| char.is_whitespace());
    }
    /// Skips forward until a whitespace character is next, and returns the area skipped over as a string slice.
    ///
    /// If a whitespace character is already next, nothing is done.
    ///
    /// Returns `None` if this slicer is past the end of the string.
    ///
    /// Opposite of [`slice_whitespace`].
    ///
    /// Equivalent to `slice_until(|char: char| char.is_whitespace())`
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a sentence.".as_slicer();
    /// assert_eq!(slicer.slice_non_whitespace(), Some("This"));
    /// ```
    ///
    /// [`slice_whitespace`]: struct.StrSlicer.html#method.slice_whitespace
    pub fn slice_non_whitespace(&mut self) -> Option<&'str str> {
        self.slice_until(|char: char| char.is_whitespace())
    }
    
    /// Skips past the rest of the line.
    ///
    /// Equivalent to `skip_until_after('\n')`
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "Line 1\nLine 2\nLine 3".as_slicer();
    /// slicer.skip_line();
    /// slicer.skip_line();
    /// assert_eq!(slicer.is_next("Line 3"), true);
    /// ```
    pub fn skip_line(&mut self) {
        self.skip_until_after('\n');
    }
    /// Skips past the rest of the line, and returns the area skipped over as a string slice.
    ///
    /// The returned string slice also has the newline characters removed, regardless of
    /// whether the line ending is `\r\n` or `\n`. It handles line endings the same way as
    /// the standard library function [`str::lines()`](https://doc.rust-lang.org/nightly/std/primitive.str.html#method.lines).
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "Line 1\nLine 2\nLine 3".as_slicer();
    /// assert_eq!(slicer.slice_line(), Some("Line 1"));
    /// assert_eq!(slicer.slice_line(), Some("Line 2"));
    /// assert_eq!(slicer.slice_line(), Some("Line 3"));
    /// ```
    pub fn slice_line(&mut self) -> Option<&'str str> {
        let line = self.slice_until_after('\n');
        line.map(|line| {
            line.trim_right_matches(|char: char| char == '\n' || char == '\r')
        })
    }

    /// Skips to the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a very long string that we just want to skip over entirely.".as_slicer();
    /// slicer.skip_to_end();
    /// assert_eq!(slicer.is_at_end(), true);
    /// ```
    pub fn skip_to_end(&mut self) {
        unsafe {
            let byte_pos = self.end_byte_pos();
            self.jump_to_unchecked(byte_pos);
        }
    }
    /// Skips to the end of the string, and returns the area skipped over as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a very long string that we just want to skip over entirely.".as_slicer();
    /// slicer.skip_until_after("skip over");
    /// assert_eq!(slicer.slice_to_end(), Some(" entirely."));
    /// ```
    pub fn slice_to_end(&mut self) -> Option<&'str str> {
        let start_pos = self.byte_pos;
        if start_pos >= self.end_byte_pos() {
            None
        } else {
            self.skip_to_end();
            let end_pos = self.byte_pos;
            Some(&self.string[start_pos..end_pos])
        }
    }
    /// Checks whether or not the string slicer is at or past the end of the string it is operating on.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// let mut slicer = "This is a very long string that we just want to skip over entirely.".as_slicer();
    /// slicer.skip_to_end();
    /// assert_eq!(slicer.is_at_end(), true);
    /// ```
    pub fn is_at_end(&self) -> bool {
        self.byte_pos >= self.end_byte_pos()
    }
}

impl<'str, T: Tracker> AsRef<str> for StrSlicer<'str, T> {
    fn as_ref(&self) -> &str {
        self.string
    }
}

/// Used by `jump_oob_fail` and `jump_char_boundary_fail`
//truncate `&str` to length at most equal to `max`,
//return `true` if it were truncated, and the new str.
//basically copied from the truncate_to_char_boundary function in libcore/str/mod.rs
fn truncate_to_char_boundary(s: &str, mut max: usize) -> (bool, &str) {
    if max >= s.len() {
        (false, s)
    } else {
        while !s.is_char_boundary(max) {
            max -= 1;
        }
        (true, &s[..max])
    }
}

/// Function that panics for out-of-bound errors in [`StrSlicer::jump_to`]
///
/// [`StrSlicer::jump_to`]: struct.StrSlicer.html#method.jump_to
//basically copied from the slice_error_fail function in libcore/str/mod.rs
#[inline(never)]
#[cold]
fn jump_oob_fail(string: &str, byte_pos: usize) -> ! {
    const MAX_DISPLAY_LENGTH: usize = 256;
    let (truncated, s_trunc) = truncate_to_char_boundary(string, MAX_DISPLAY_LENGTH);
    let ellipsis = if truncated { "[...]" } else { "" };

    panic!("byte index {} is out of bounds of `{}`{}", byte_pos, s_trunc, ellipsis);
}

/// Function that panics for jumping to a position that is not a UTF-8
/// char boundary in [`StrSlicer::jump_to`]
///
/// [`StrSlicer::jump_to`]: struct.StrSlicer.html#method.jump_to
//basically copied from the slice_error_fail function in libcore/str/mod.rs
#[inline(never)]
#[cold]
fn jump_char_boundary_fail(string: &str, byte_pos: usize) -> ! {
    const MAX_DISPLAY_LENGTH: usize = 256;
    let (truncated, s_trunc) = truncate_to_char_boundary(string, MAX_DISPLAY_LENGTH);
    let ellipsis = if truncated { "[...]" } else { "" };
    
    //find the start index of the character byte_pos is inside of
    let mut char_start = byte_pos;
    while !string.is_char_boundary(char_start) {
        char_start -= 1;
    }
    
    //`char_start` must be less than len and a char boundary
    let char = string[char_start..].chars().next().unwrap();
    let char_byte_range = char_start..(char_start + char.len_utf8());
    
    panic!("byte index {} is not a char boundary; it is inside {:?} (bytes {:?}) of `{}`{}",
           byte_pos, char, char_byte_range, s_trunc, ellipsis);
}

/// A module containing various [`Tracker`] types.
///
/// [`Tracker`]: trait.Tracker.html
pub mod trackers {
    use ::Tracker;
    
    const NEWLINE: char = '\n';
    
    /// A [`Tracker`] that tracks the line number.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicer::AsSlicer;
    /// use slicer::trackers::LineTracker;
    ///
    /// let mut slicer = "Line 1\nLine 2\nLine 3".as_slicer_with_tracker(LineTracker::new());
    /// slicer.skip_line(); //skip over line 0
    /// assert_eq!(slicer.tracker_pos(), 1); //it is currently on line 1
    /// ```
    ///
    /// [`Tracker`]: ../trait.Tracker.html
    #[derive(Debug, Clone)]
    pub struct LineTracker {
        lines: usize,
        line_byte_pos: usize
    }
    impl LineTracker {
        pub fn new() -> Self {
            Self {
                lines: 0,
                line_byte_pos: 0
            }
        }
        /// Returns the line number. The same as this type's implementation of the [`Tracker::pos`] method.
        ///
        /// [`Tracker::pos`]: ../trait.Tracker.html#tymethod.pos
        #[inline]
        pub fn lines(&self) -> usize {
            self.lines
        }
        /// Returns byte index of the start of the current line.
        #[inline]
        pub fn line_byte_pos(&self) -> usize {
            self.line_byte_pos
        }
    }
    impl Default for LineTracker {
        fn default() -> Self {
            Self::new()
        }
    }
    impl Tracker for LineTracker {
        type Pos = usize;
        fn pos(&self) -> Self::Pos {
            self.lines
        }
        fn update(&mut self, string: &str, old_byte_pos: usize, new_byte_pos: usize) {
            
            //if we're jumping forward, simply add up the newlines in the area we're jumping through
            if new_byte_pos > old_byte_pos {
                
                let mut newline_count = 0;
                for (index, _) in string[old_byte_pos..new_byte_pos].match_indices(NEWLINE) {
                    newline_count += 1;
                    self.line_byte_pos = index;
                }
                self.lines += newline_count;
                
            //if we're jumping backwards, we either start over and count the number of newlines
            //from the beginning, or subtract newlines, depending on how far the point we've jumped to
            //is from the start
            } else if new_byte_pos < old_byte_pos {
                
                let diff = old_byte_pos - new_byte_pos;
                let half_len_to_root = old_byte_pos / 2;
                
                if diff > half_len_to_root {
                    
                    let mut newline_count = 0;
                    for (index, _) in string[0..new_byte_pos].match_indices(NEWLINE) {
                        newline_count += 1;
                        self.line_byte_pos = index;
                    }
                    self.lines = newline_count;
                    
                } else {
                    
                    let mut newline_count = 0;
                    for (index, _) in string[new_byte_pos..old_byte_pos].match_indices(NEWLINE) {
                        newline_count += 1;
                        self.line_byte_pos = index;
                    }
                    self.lines -= newline_count;
                    
                }
            }
            
        }
    }
}